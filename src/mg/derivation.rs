//! Derivation trees for Minimalist Grammar

use std::fmt;
use std::hash::Hash;
use crate::mg::feature::Feature;
use crate::mg::lexical_item::LexicalItem;
use crate::common::FeatureStructure;
use crate::common::ParseNode;

/// Chain elements in a derived structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chain {
    /// The lexical item at the head of the chain
    pub head: LexicalItem,
    /// The tail positions (traces)
    pub tail: Vec<usize>,
    /// Agreement information
    pub agreement: Option<FeatureStructure>,
    /// Whether this is a phase head
    pub is_phase_head: bool,
}

impl Chain {
    /// Create a new chain with just a head
    pub fn new(head: LexicalItem) -> Self {
        // Check if this is a phase head
        let is_phase_head = head.is_phase_head();
        
        // Extract agreement features from the head
        let agreement = head.agreement_features.clone();
        
        Chain {
            head,
            tail: Vec::new(),
            agreement,
            is_phase_head,
        }
    }
    
    /// Create a chain with a head and tail positions
    pub fn with_tail(head: LexicalItem, tail: Vec<usize>) -> Self {
        let mut chain = Self::new(head);
        chain.tail = tail;
        chain
    }
    
    /// Create a chain with explicit agreement information
    pub fn with_agreement(mut self, agreement: FeatureStructure) -> Self {
        self.agreement = Some(agreement);
        self
    }
    
    /// Merge agreement information from another chain
    pub fn merge_agreement(&mut self, other: &Chain) {
        if let Some(other_agr) = &other.agreement {
            if let Some(self_agr) = &mut self.agreement {
                if let Some(merged) = self_agr.unify(other_agr) {
                    self.agreement = Some(merged);
                }
            } else {
                self.agreement = Some(other_agr.clone());
            }
        }
    }
    
    /// Check if this chain has any tail positions (traces)
    pub fn has_traces(&self) -> bool {
        !self.tail.is_empty()
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.head)?;
        
        if !self.tail.is_empty() {
            write!(f, " (traces: {:?})", self.tail)?;
        }
        
        Ok(())
    }
}

/// Derived syntactic structure in Minimalist Grammar
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DerivationTree {
    /// The chain at this node
    pub chain: Chain,
    /// Children (left, right) - binary branching
    pub children: Option<(Box<DerivationTree>, Box<DerivationTree>)>,
    /// Indexes for linearization
    pub index: usize,
    /// Whether this is an adjunct structure (for pair merge)
    pub is_adjunct: bool,
    /// Delayed features for late merge
    pub delayed_features: Vec<Feature>,
    /// Whether this derivation is a phase
    pub is_phase: bool,
    /// Whether the phase is completed (transferred to interfaces)
    pub phase_completed: bool,
}

impl DerivationTree {
    /// Create a new leaf node
    pub fn leaf(item: LexicalItem, index: usize) -> Self {
        // Check if this is a phase
        let is_phase = item.is_phase_head();
        
        // Extract any delayed features
        let delayed_features = item.get_delayed_features();
        
        DerivationTree {
            chain: Chain::new(item),
            children: None,
            index,
            is_adjunct: false,
            delayed_features,
            is_phase,
            phase_completed: false,
        }
    }
    
    /// Create a new internal node via Merge
    pub fn merge(left: DerivationTree, right: DerivationTree, head_features: Vec<Feature>, index: usize) -> Self {
        // Create a new lexical item with the remaining features
        let head_item = LexicalItem {
            phonetic_form: if !left.chain.head.phonetic_form.is_empty() {
                left.chain.head.phonetic_form.clone()
            } else {
                right.chain.head.phonetic_form.clone()
            },
            features: head_features.clone(),
            agreement_features: None,
        };
        
        // Extract any delayed features
        let delayed_features: Vec<Feature> = head_features.iter()
            .filter_map(|f| {
                if let Feature::Delayed(inner) = f {
                    Some(*inner.clone())
                } else {
                    None
                }
            })
            .collect();
        
        // Check if this is a phase
        let is_phase = head_features.iter().any(|f| f.is_phase_head());
        
        // Create the merged node
        let mut chain = Chain::new(head_item);
        
        // Merge agreement features from both children
        chain.merge_agreement(&left.chain);
        chain.merge_agreement(&right.chain);
        
        DerivationTree {
            chain,
            children: Some((Box::new(left), Box::new(right))),
            index,
            is_adjunct: false,
            delayed_features,
            is_phase,
            phase_completed: false,
        }
    }
    
    /// Create a new internal node via Pair Merge (adjunction)
    pub fn pair_merge(host: DerivationTree, adjunct: DerivationTree, index: usize) -> Self {
        // In pair merge, the host projects and the adjunct becomes an adjunct
        let mut result = host.clone();
        
        // Mark the adjunct as an adjunct
        let mut adjunct_copy = adjunct.clone();
        adjunct_copy.is_adjunct = true;
        
        // Combine them - adjunct as the specifier (left child)
        result.children = Some((Box::new(adjunct_copy), Box::new(host)));
        result.index = index;
        
        result
    }
    
    /// Create a new node via Late Merge
    pub fn late_merge(host: DerivationTree, delayed_material: DerivationTree, index: usize) -> Self {
        // Similar to regular merge, but applied to already-moved material
        let mut result = host.clone();
        
        // Use the delayed features that were saved
        if !result.delayed_features.is_empty() {
            let feature = result.delayed_features.remove(0);
            
            // Check if the delayed material matches the expected feature
            if let Some(first_feature) = delayed_material.first_feature() {
                match (&feature, first_feature) {
                    (Feature::Selector(sel), Feature::Categorial(cat)) => {
                        if sel == cat {
                            // Can apply late merge - attach the delayed material appropriately
                            result.children = Some((Box::new(delayed_material), Box::new(host)));
                            result.index = index;
                        }
                    },
                    _ => {}
                }
            }
        }
        
        result
    }
    
    /// Create a new node via Move
    pub fn r#move(base: DerivationTree, moved_chain: Chain, head_features: Vec<Feature>, index: usize) -> Self {
        // Extract any delayed features
        let delayed_features: Vec<Feature> = head_features.iter()
            .filter_map(|f| {
                if let Feature::Delayed(inner) = f {
                    Some(*inner.clone())
                } else {
                    None
                }
            })
            .collect();
        
        // Check if this is a phase
        let is_phase = head_features.iter().any(|f| f.is_phase_head());
        
        DerivationTree {
            chain: Chain {
                head: LexicalItem {
                    phonetic_form: moved_chain.head.phonetic_form.clone(),
                    features: head_features,
                    agreement_features: moved_chain.agreement.clone(),
                },
                tail: moved_chain.tail,
                agreement: moved_chain.agreement,
                is_phase_head: moved_chain.is_phase_head,
            },
            children: Some((Box::new(base), Box::new(DerivationTree {
                chain: Chain::new(LexicalItem::empty()),
                children: None,
                index: 0, // Traces don't need unique indexes
                is_adjunct: false,
                delayed_features: Vec::new(),
                is_phase: false,
                phase_completed: false,
            }))),
            index,
            is_adjunct: false,
            delayed_features,
            is_phase,
            phase_completed: false,
        }
    }
    
    /// Get the first feature of this node's chain head
    pub fn first_feature(&self) -> Option<&Feature> {
        self.chain.head.first_feature()
    }
    
    /// Remove the first feature from this node's chain head
    pub fn remove_first_feature(&mut self) -> Option<Feature> {
        self.chain.head.remove_first_feature()
    }
    
    /// Create a copy with the first feature removed
    pub fn without_first_feature(&self) -> Self {
        let mut result = self.clone();
        result.remove_first_feature();
        result
    }
    
    /// Check if this is a phase
    pub fn is_phase(&self) -> bool {
        self.is_phase
    }
    
    /// Mark this phase as completed (transferred to interfaces)
    pub fn complete_phase(&mut self) {
        if self.is_phase {
            self.phase_completed = true;
        }
    }
    
    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }
    
    /// Calculate the depth of this derivation tree
    pub fn depth(&self) -> usize {
        if let Some((left, right)) = &self.children {
            1 + std::cmp::max(left.depth(), right.depth())
        } else {
            0
        }
    }
    
    /// Get the yield (linearized string) of this tree
    pub fn get_yield(&self) -> Vec<String> {
        // Return the linearized string
        let mut forms = Vec::new();
        
        // Add this node's phonetic form if non-empty and not a trace
        if !self.chain.head.phonetic_form.is_empty() && !self.chain.tail.contains(&self.index) {
            forms.push(self.chain.head.phonetic_form.clone());
        }
        
        // Recursively collect from children
        if let Some((left, right)) = &self.children {
            forms.extend(left.as_ref().get_yield());
            forms.extend(right.as_ref().get_yield());
        }
        
        forms
    }
}

impl fmt::Display for DerivationTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_tree(node: &DerivationTree, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let indent_str = " ".repeat(indent);
            
            // Print the current node
            write!(f, "{}{}", indent_str, node.chain)?;
            if node.is_adjunct {
                write!(f, " (adjunct)")?;
            }
            if node.is_phase {
                write!(f, " (phase")?;
                if node.phase_completed {
                    write!(f, ", completed")?;
                }
                write!(f, ")")?;
            }
            writeln!(f)?;
            
            // Print children recursively
            if let Some((left, right)) = &node.children {
                print_tree(left, indent + 2, f)?;
                print_tree(right, indent + 2, f)?;
            }
            
            Ok(())
        }
        
        print_tree(self, 0, f)
    }
}

impl ParseNode for DerivationTree {
    type Cat = LexicalItem;
    
    fn category(&self) -> &Self::Cat {
        &self.chain.head
    }
    
    fn word(&self) -> Option<&str> {
        if self.chain.head.phonetic_form.is_empty() {
            None
        } else {
            Some(&self.chain.head.phonetic_form)
        }
    }
    
    fn children(&self) -> &[Self] {
        // This is tricky since we have Option<(Box<Self>, Box<Self>)>
        // We'll need to return an empty slice for now
        &[]
    }
    
    fn rule(&self) -> Option<&str> {
        // Return the rule type that created this node
        if self.is_adjunct {
            Some("Adjunction")
        } else if self.chain.has_traces() {
            Some("Move")
        } else if self.children.is_some() {
            Some("Merge")
        } else {
            Some("Lexical")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::FeatureValue;
    
    #[test]
    fn test_chain_creation() {
        let item = LexicalItem::new("the", vec![
            Feature::Categorial("D".to_string()),
        ]);
        
        let chain = Chain::new(item.clone());
        
        assert_eq!(chain.head, item);
        assert!(chain.tail.is_empty());
        assert!(!chain.is_phase_head);
        
        // Test with tail
        let chain_with_tail = Chain::with_tail(item, vec![1, 2, 3]);
        assert_eq!(chain_with_tail.tail, vec![1, 2, 3]);
        assert!(chain_with_tail.has_traces());
        
        // Test with agreement
        let mut agr = FeatureStructure::new();
        agr.add("num", FeatureValue::Atomic("sg".to_string()));
        
        let chain_with_agr = Chain::new(LexicalItem::new("he", vec![
            Feature::Categorial("D".to_string()),
        ])).with_agreement(agr.clone());
        
        assert!(chain_with_agr.agreement.is_some());
        assert_eq!(chain_with_agr.agreement.unwrap().get("num"), 
                  Some(&FeatureValue::Atomic("sg".to_string())));
    }
    
    #[test]
    fn test_derivation_tree_creation() {
        // Create some lexical items
        let det = LexicalItem::new("the", vec![
            Feature::Categorial("D".to_string()),
        ]);
        
        let noun = LexicalItem::new("cat", vec![
            Feature::Categorial("N".to_string()),
        ]);
        
        // Create leaf nodes
        let det_node = DerivationTree::leaf(det, 0);
        let noun_node = DerivationTree::leaf(noun, 1);
        
        assert!(det_node.is_leaf());
        assert_eq!(det_node.depth(), 0);
        
        // Test merge
        let dp_node = DerivationTree::merge(
            det_node, 
            noun_node, 
            vec![Feature::Categorial("DP".to_string())],
            2
        );
        
        assert!(!dp_node.is_leaf());
        assert_eq!(dp_node.depth(), 1);
        assert_eq!(dp_node.index, 2);
        
        if let Some((left, right)) = &dp_node.children {
            assert_eq!(left.chain.head.phonetic_form, "the");
            assert_eq!(right.chain.head.phonetic_form, "cat");
        } else {
            panic!("Expected children");
        }
    }
    
    #[test]
    fn test_move_operation() {
        // Create a base structure
        let v = LexicalItem::new("sees", vec![
            Feature::Categorial("v".to_string()),
            Feature::Licensor("wh".to_string()),
        ]);
        
        let dp = LexicalItem::new("what", vec![
            Feature::Categorial("D".to_string()),
            Feature::Licensee("wh".to_string()),
        ]);
        
        let v_node = DerivationTree::leaf(v, 0);
        let dp_node = DerivationTree::leaf(dp, 1);
        
        // Merge them
        let base = DerivationTree::merge(
            dp_node,
            v_node,
            vec![Feature::Categorial("vP".to_string())],
            2
        );
        
        // Now move the DP (wh-movement)
        let dp_chain = Chain::with_tail(
            LexicalItem::new("what", vec![
                Feature::Categorial("D".to_string()),
            ]),
            vec![1]
        );
        
        let moved = DerivationTree::r#move(
            base,
            dp_chain,
            vec![Feature::Categorial("CP".to_string())],
            3
        );
        
        // Check the resulting structure
        assert_eq!(moved.chain.head.phonetic_form, "what");
        assert_eq!(moved.chain.tail, vec![1]);
        assert_eq!(moved.chain.head.features[0], Feature::Categorial("CP".to_string()));
        
        if let Some((base_child, trace_child)) = &moved.children {
            assert_eq!(base_child.chain.head.phonetic_form, "sees");
            assert!(trace_child.chain.head.is_empty());
        } else {
            panic!("Expected children");
        }
    }
    
    #[test]
    fn test_pair_merge() {
        // Create a noun and adjective
        let noun = LexicalItem::new("book", vec![
            Feature::Categorial("N".to_string()),
        ]);
        
        let adj = LexicalItem::new("red", vec![
            Feature::Categorial("A".to_string()),
        ]);
        
        let noun_node = DerivationTree::leaf(noun, 0);
        let adj_node = DerivationTree::leaf(adj, 1);
        
        // Pair merge (adjunction)
        let result = DerivationTree::pair_merge(noun_node, adj_node, 2);
        
        // Check the result
        assert_eq!(result.chain.head.phonetic_form, "book");
        
        if let Some((left, right)) = &result.children {
            assert_eq!(left.chain.head.phonetic_form, "red");
            assert!(left.is_adjunct);
            assert_eq!(right.chain.head.phonetic_form, "book");
            assert!(!right.is_adjunct);
        } else {
            panic!("Expected children");
        }
    }
    
    #[test]
    fn test_late_merge() {
        // Create a DP with a delayed feature
        let d = LexicalItem::new("the", vec![
            Feature::Categorial("D".to_string()),
            Feature::Delayed(Box::new(Feature::Selector("N".to_string()))),
        ]);
        
        // Create a noun for late merger
        let n = LexicalItem::new("book", vec![
            Feature::Categorial("N".to_string()),
        ]);
        
        let dp_node = DerivationTree::leaf(d, 0);
        let n_node = DerivationTree::leaf(n, 1);
        
        // Late merge
        let result = DerivationTree::late_merge(dp_node, n_node, 2);
        
        // Check that the delayed feature was used
        assert!(result.delayed_features.is_empty());
        
        if let Some((left, right)) = &result.children {
            assert_eq!(left.chain.head.phonetic_form, "book");
            assert_eq!(right.chain.head.phonetic_form, "the");
        } else {
            panic!("Expected children");
        }
    }
    
    #[test]
    fn test_phase_operations() {
        // Create a phase head
        let c = LexicalItem::new("that", vec![
            Feature::Categorial("C".to_string()),
            Feature::Phase("C".to_string()),
        ]);
        
        let c_node = DerivationTree::leaf(c, 0);
        
        // Check phase properties
        assert!(c_node.is_phase());
        assert!(!c_node.phase_completed);
        
        // Mark phase as completed
        let mut c_node_completed = c_node.clone();
        c_node_completed.complete_phase();
        
        assert!(c_node_completed.phase_completed);
    }
}