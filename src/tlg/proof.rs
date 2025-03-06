//! Natural deduction proof trees for Type-Logical Grammar

use std::fmt;
use crate::tlg::logical_type::LogicalType;
use crate::common::ParseNode;

/// Labeled natural deduction proof node for Type-Logical Grammar
#[derive(Debug, Clone)]
pub struct ProofNode {
    /// The logical type
    pub logical_type: LogicalType,
    /// The label for natural deduction (λ-term)
    pub label: String,
    /// Children in the proof tree
    pub children: Vec<ProofNode>,
    /// The inference rule used
    pub rule: Option<String>,
}

impl ProofNode {
    /// Create a new axiom (leaf) node
    pub fn axiom(label: &str, logical_type: LogicalType) -> Self {
        ProofNode {
            logical_type,
            label: label.to_string(),
            children: vec![],
            rule: None,
        }
    }

    /// Create a new internal node in the proof tree
    pub fn infer(logical_type: LogicalType, children: Vec<ProofNode>, rule: &str) -> Self {
        // For non-axioms, generate a composite label derived from children
        let label = Self::generate_label(&children, rule);
        
        ProofNode {
            logical_type,
            label,
            children,
            rule: Some(rule.to_string()),
        }
    }

    /// Generate a label for a proof node based on its children and rule
    fn generate_label(children: &[ProofNode], rule: &str) -> String {
        match rule {
            "→E" => {
                // Function application: combine function and argument labels
                if children.len() == 2 {
                    format!("{}({})", children[0].label, children[1].label)
                } else {
                    "invalid".to_string()
                }
            },
            "→I" => {
                // Lambda abstraction: λx.M
                if !children.is_empty() {
                    let var = children[0].label.chars().next().unwrap_or('x');
                    format!("λ{}.{}", var, children[0].label)
                } else {
                    "invalid".to_string()
                }
            },
            "⊗E" => {
                // Product elimination: pair destructuring
                if children.len() >= 2 {
                    format!("let ({},{}) = {} in {}", 
                           children[0].label.chars().next().unwrap_or('x'),
                           children[0].label.chars().nth(1).unwrap_or('y'),
                           children[1].label,
                           children[0].label)
                } else {
                    "invalid".to_string()
                }
            },
            "⊗I" => {
                // Product introduction: pair construction
                if children.len() == 2 {
                    format!("({},{})", children[0].label, children[1].label)
                } else {
                    "invalid".to_string()
                }
            },
            "◇E" => {
                // Diamond elimination
                if children.len() == 2 {
                    format!("let◇{} = {} in {}", 
                           children[0].label.chars().next().unwrap_or('x'),
                           children[1].label,
                           children[0].label)
                } else {
                    "invalid".to_string()
                }
            },
            "◇I" => {
                // Diamond introduction
                if !children.is_empty() {
                    format!("◇{}", children[0].label)
                } else {
                    "invalid".to_string()
                }
            },
            "□E" => {
                // Box elimination
                if !children.is_empty() {
                    format!("unbox({})", children[0].label)
                } else {
                    "invalid".to_string()
                }
            },
            "□I" => {
                // Box introduction
                if !children.is_empty() {
                    format!("box({})", children[0].label)
                } else {
                    "invalid".to_string()
                }
            },
            "←E" => {
                // Backward application: combine function and argument labels
                if children.len() == 2 {
                    format!("{}({})", children[0].label, children[1].label)
                } else {
                    "invalid".to_string()
                }
            },
            _ => {
                // Default case for other rules
                let mut combined = String::new();
                for (i, child) in children.iter().enumerate() {
                    if i > 0 {
                        combined.push('_');
                    }
                    combined.push_str(&child.label);
                }
                if combined.is_empty() {
                    rule.to_string()
                } else {
                    combined
                }
            }
        }
    }
    
    /// Get the depth of this proof tree
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            1 + self.children.iter().map(|child| child.depth()).max().unwrap_or(0)
        }
    }
    
    /// Get the number of nodes in this proof tree
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|child| child.node_count()).sum::<usize>()
    }
    
    /// Check if this proof uses a particular rule
    pub fn uses_rule(&self, rule_name: &str) -> bool {
        if let Some(r) = &self.rule {
            if r == rule_name {
                return true;
            }
        }
        
        for child in &self.children {
            if child.uses_rule(rule_name) {
                return true;
            }
        }
        
        false
    }
}

impl fmt::Display for ProofNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_tree(node: &ProofNode, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let indent_str = " ".repeat(indent);
            
            write!(f, "{}{} : {}", indent_str, node.label, node.logical_type)?;
            
            if let Some(rule) = &node.rule {
                write!(f, " [{}]", rule)?;
            }
            
            writeln!(f)?;
            
            for child in &node.children {
                print_tree(child, indent + 2, f)?;
            }
            
            Ok(())
        }
        
        print_tree(self, 0, f)
    }
}

/// A state in the proof search for Type-Logical Grammar
#[derive(Debug, Clone)]
pub struct ProofSearchState {
    /// The current sequent items
    pub items: Vec<ProofNode>,
    /// The history of rules applied so far
    pub rule_history: Vec<String>,
    /// The depth of the search
    pub depth: usize,
}

impl ProofSearchState {
    /// Create a new initial search state
    pub fn new(axioms: Vec<ProofNode>) -> Self {
        Self {
            items: axioms,
            rule_history: vec![],
            depth: 0,
        }
    }
    
    /// Apply a rule and generate a new state
    pub fn apply_rule(&self, rule_name: &str, result: ProofNode, 
                     used_indices: Vec<usize>) -> ProofSearchState {
        let mut new_items = Vec::new();
        let mut used_indices_sorted = used_indices.clone();
        used_indices_sorted.sort_unstable();
        used_indices_sorted.reverse(); // Remove from end to not invalidate indices
        
        // Copy items except for the used ones
        for (i, item) in self.items.iter().enumerate() {
            if !used_indices.contains(&i) {
                new_items.push(item.clone());
            }
        }
        
        // Add the result
        new_items.push(result);
        
        // Update history
        let mut new_history = self.rule_history.clone();
        new_history.push(rule_name.to_string());
        
        ProofSearchState {
            items: new_items,
            rule_history: new_history,
            depth: self.depth + 1,
        }
    }
    
    /// Check if this state is a complete proof with the target logical type
    pub fn is_complete(&self, target: &LogicalType) -> bool {
        self.items.len() == 1 && &self.items[0].logical_type == target
    }
    
    /// Get the current proof if this state is complete
    pub fn get_proof(&self) -> Option<ProofNode> {
        if self.items.len() == 1 {
            Some(self.items[0].clone())
        } else {
            None
        }
    }
}

/// Implement ParseNode trait for ProofNode to integrate with common interfaces
impl ParseNode for ProofNode {
    type Cat = LogicalType;
    
    fn category(&self) -> &Self::Cat {
        &self.logical_type
    }
    
    fn word(&self) -> Option<&str> {
        None // ProofNode doesn't directly have word information
    }
    
    fn children(&self) -> &[Self] {
        &self.children
    }
    
    fn rule(&self) -> Option<&str> {
        self.rule.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_axiom_creation() {
        let np = LogicalType::np();
        let axiom = ProofNode::axiom("john", np.clone());
        
        assert_eq!(axiom.label, "john");
        assert_eq!(axiom.logical_type, np);
        assert!(axiom.children.is_empty());
        assert_eq!(axiom.rule, None);
    }
    
    #[test]
    fn test_infer_creation() {
        let np = LogicalType::np();
        let s = LogicalType::s();
        let verb_type = LogicalType::left_impl(s.clone(), np.clone());
        
        let john = ProofNode::axiom("john", np.clone());
        let sleeps = ProofNode::axiom("sleeps", verb_type.clone());
        
        let inferred = ProofNode::infer(
            s.clone(),
            vec![sleeps, john],
            "←E"
        );
        
        assert_eq!(inferred.logical_type, s);
        assert_eq!(inferred.children.len(), 2);
        assert_eq!(inferred.rule, Some("←E".to_string()));
    }
    
    #[test]
    fn test_proof_search_state() {
        let np = LogicalType::np();
        let s = LogicalType::s();
        let verb_type = LogicalType::left_impl(s.clone(), np.clone());
        
        let john = ProofNode::axiom("john", np.clone());
        let sleeps = ProofNode::axiom("sleeps", verb_type.clone());
        
        let state = ProofSearchState::new(vec![john, sleeps]);
        
        assert_eq!(state.items.len(), 2);
        assert!(state.rule_history.is_empty());
        assert_eq!(state.depth, 0);
        
        let combined = ProofNode::infer(
            s.clone(),
            vec![state.items[1].clone(), state.items[0].clone()],
            "←E"
        );
        
        let new_state = state.apply_rule("←E", combined, vec![0, 1]);
        
        assert_eq!(new_state.items.len(), 1);
        assert_eq!(new_state.rule_history, vec!["←E".to_string()]);
        assert_eq!(new_state.depth, 1);
        assert!(new_state.is_complete(&s));
        assert!(new_state.get_proof().is_some());
    }
}