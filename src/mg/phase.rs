//! Phase-based computation for Minimalist Grammar

use crate::mg::derivation::DerivationTree;
use crate::mg::feature::Feature;

/// Phase configuration for minimalist grammar
#[derive(Debug, Clone)]
pub struct PhaseConfig {
    /// Whether to enforce the Phase Impenetrability Condition (PIC)
    pub enforce_pic: bool,
    /// Elements that count as phase heads
    pub phase_heads: Vec<String>,
    /// Maximum number of elements that can be at the phase edge
    pub max_edge_elements: usize,
    /// Whether to transfer phases immediately when completed
    pub immediate_transfer: bool,
}

impl Default for PhaseConfig {
    fn default() -> Self {
        Self {
            enforce_pic: true,
            phase_heads: vec!["C".to_string(), "v".to_string(), "D".to_string()],
            max_edge_elements: 1,
            immediate_transfer: true,
        }
    }
}

/// Phase impenetrability checker
#[derive(Clone)]
pub struct PhaseChecker {
    config: PhaseConfig,
}

impl PhaseChecker {
    /// Create a new phase checker with the specified configuration
    pub fn new(config: PhaseConfig) -> Self {
        Self { config }
    }
    
    /// Check if a node is a phase head
    pub fn is_phase_head(&self, node: &DerivationTree) -> bool {
        // Check if the node has a Phase feature
        if node.chain.head.features.iter().any(|f| matches!(f, Feature::Phase(_))) {
            return true;
        }
        
        // Check if the node is a categorial feature that is a phase head
        if let Some(Feature::Categorial(cat)) = node.first_feature() {
            return self.config.phase_heads.contains(cat);
        }
        
        false
    }
    
    /// Get phase edge elements of a phase
    pub fn get_phase_edge<'a>(&self, phase: &'a DerivationTree) -> Vec<&'a DerivationTree> {
        let mut edge_elements = Vec::new();
        
        // If this isn't a phase, return empty
        if !self.is_phase_head(phase) {
            return edge_elements;
        }
        
        // Find specifiers (elements in the left periphery)
        if let Some((left, _)) = &phase.children {
            self.collect_specifiers(left, &mut edge_elements);
        }
        
        // Limit to max edge elements
        if edge_elements.len() > self.config.max_edge_elements {
            edge_elements.truncate(self.config.max_edge_elements);
        }
        
        edge_elements
    }
    
    /// Collect specifiers for the phase edge
    fn collect_specifiers<'a>(&self, node: &'a DerivationTree, elements: &mut Vec<&'a DerivationTree>) {
        // Add this node if it's a specifier
        elements.push(node);
        
        // If there are more specifiers, collect those too
        if let Some((left, _)) = &node.children {
            self.collect_specifiers(left, elements);
        }
    }
    
    /// Check if extraction from a phase is valid according to PIC
    pub fn check_extraction(&self, phase: &DerivationTree, target_index: usize) -> bool {
        // If PIC is not enforced, always allow extraction
        if !self.config.enforce_pic {
            return true;
        }
        
        // If the phase is not completed, allow extraction
        if !phase.phase_completed {
            return true;
        }
        
        // Check if the target is at the phase edge
        let edge_elements = self.get_phase_edge(phase);
        
        for element in edge_elements {
            // If the element itself has the index
            if element.index == target_index {
                return true;
            }
            
            // If the element contains the index in its chain
            if element.chain.tail.contains(&target_index) {
                return true;
            }
        }
        
        // Target is not at phase edge, extraction is blocked
        false
    }
    
    /// Transfer a completed phase to the interfaces
    /// 
    /// In minimalist theory, this means making the complement of the phase head 
    /// inaccessible for further syntactic operations (except through the phase edge)
    pub fn transfer_phase(&self, tree: &mut DerivationTree) {
        if !self.is_phase_head(tree) {
            return;
        }
        
        tree.complete_phase();
        
        // Recursively transfer any embedded phases
        if let Some((left, right)) = &mut tree.children {
            self.transfer_phase(left);
            self.transfer_phase(right);
        }
    }
    
    /// Calculate the phase spine (path of phase heads) in a derivation
    pub fn phase_spine<'a>(&self, tree: &'a DerivationTree) -> Vec<&'a DerivationTree> {
        let mut spine = Vec::new();
        self.calculate_phase_spine(tree, &mut spine);
        spine
    }
    
    fn calculate_phase_spine<'a>(&self, node: &'a DerivationTree, spine: &mut Vec<&'a DerivationTree>) {
        if self.is_phase_head(node) {
            spine.push(node);
        }
        
        // Continue down the complement
        if let Some((_, right)) = &node.children {
            self.calculate_phase_spine(right, spine);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mg::lexical_item::LexicalItem;
    // Helper function to create a simple CP phase
    fn create_cp_phase() -> DerivationTree {
        // Create C (phase head)
        let c = LexicalItem::new("that", vec![
            Feature::Categorial("C".to_string()),
            Feature::Phase("C".to_string()), // Explicit phase marking
        ]);
        
        // Create a DP (specifier)
        let dp = LexicalItem::new("John", vec![
            Feature::Categorial("D".to_string()),
        ]);
        
        // Create a TP (complement)
        let tp = LexicalItem::new("", vec![
            Feature::Categorial("T".to_string()),
        ]);
        
        // Create the specifier and complement nodes
        let dp_node = DerivationTree::leaf(dp, 1);
        let _tp_node = DerivationTree::leaf(tp, 2);
        
        // Merge to create [DP [C TP]]
        DerivationTree::merge(
            dp_node, 
            DerivationTree::leaf(c, 0),
            vec![Feature::Categorial("C".to_string())],
            3
        )
    }
    
    // Helper function to create a more complex structure with embedded phases
    fn create_complex_structure() -> DerivationTree {
        // Create C (phase head)
        let c = LexicalItem::new("that", vec![
            Feature::Categorial("C".to_string()),
            Feature::Phase("C".to_string()),
        ]);
        
        // Create v (phase head)
        let v = LexicalItem::new("", vec![
            Feature::Categorial("v".to_string()),
            Feature::Phase("v".to_string()),
        ]);
        
        // Create a DP (subject)
        let dp_subj = LexicalItem::new("John", vec![
            Feature::Categorial("D".to_string()),
        ]);
        
        // Create a DP (object)
        let dp_obj = LexicalItem::new("Mary", vec![
            Feature::Categorial("D".to_string()),
        ]);
        
        // Create a V
        let verb = LexicalItem::new("sees", vec![
            Feature::Categorial("V".to_string()),
        ]);
        
        // Build [DP_obj V]
        let vp = DerivationTree::merge(
            DerivationTree::leaf(dp_obj, 0),
            DerivationTree::leaf(verb, 1),
            vec![Feature::Categorial("V".to_string())],
            2
        );
        
        // Build [v [DP_obj V]]
        let v_vp = DerivationTree::merge(
            vp,
            DerivationTree::leaf(v, 3),
            vec![Feature::Categorial("v".to_string())],
            4
        );
        
        // Build [DP_subj [v [DP_obj V]]]
        let vp_with_subj = DerivationTree::merge(
            DerivationTree::leaf(dp_subj, 5),
            v_vp,
            vec![Feature::Categorial("vP".to_string())],
            6
        );
        
        // Build [C [DP_subj [v [DP_obj V]]]]
        DerivationTree::merge(
            vp_with_subj,
            DerivationTree::leaf(c, 7),
            vec![Feature::Categorial("CP".to_string())],
            8
        )
    }
    
    #[test]
    fn test_is_phase_head() {
        let config = PhaseConfig::default();
        let checker = PhaseChecker::new(config);
        
        let cp = create_cp_phase();
        
        assert!(checker.is_phase_head(&cp));
        
        // Create a non-phase head
        let dp = LexicalItem::new("John", vec![
            Feature::Categorial("D".to_string()),
            // No phase feature, and not in default phase heads for this test
        ]);
        
        let dp_node = DerivationTree::leaf(dp, 0);
        
        // Create a custom checker that doesn't include D as a phase head
        let custom_config = PhaseConfig {
            phase_heads: vec!["C".to_string(), "v".to_string()], // D is not a phase head
            ..PhaseConfig::default()
        };
        let custom_checker = PhaseChecker::new(custom_config);
        
        assert!(!custom_checker.is_phase_head(&dp_node));
    }
    
    #[test]
    fn test_phase_edge() {
        let config = PhaseConfig::default();
        let checker = PhaseChecker::new(config);
        
        let cp = create_cp_phase();
        
        let edge = checker.get_phase_edge(&cp);
        assert_eq!(edge.len(), 1); // Just the DP specifier
        
        if let Some((left, _)) = &cp.children {
            assert_eq!(edge[0].index, left.index);
        } else {
            panic!("Expected children in CP phase");
        }
    }
    
    #[test]
    fn test_extraction() {
        let config = PhaseConfig::default();
        let checker = PhaseChecker::new(config);
        
        let mut cp = create_cp_phase();
        
        // Extraction should work (phase not completed)
        assert!(checker.check_extraction(&cp, 2)); // Trying to extract the TP
        
        // Now complete the phase
        cp.complete_phase();
        
        // Extraction of complement should fail (PIC violation)
        assert!(!checker.check_extraction(&cp, 2));
        
        // Extraction of specifier should succeed (phase edge)
        assert!(checker.check_extraction(&cp, 1));
    }
    
    #[test]
    fn test_phase_spine() {
        let config = PhaseConfig::default();
        let checker = PhaseChecker::new(config);
        
        let complex = create_complex_structure();
        
        let spine = checker.phase_spine(&complex);
        assert_eq!(spine.len(), 2); // C and v
        
        // First should be CP
        assert_eq!(spine[0].chain.head.features[0], Feature::Categorial("C".to_string()));
        
        // Second should be vP
        assert_eq!(spine[1].chain.head.features[0], Feature::Categorial("v".to_string()));
    }
    
    #[test]
    fn test_phase_transfer() {
        let config = PhaseConfig {
            immediate_transfer: true,
            ..PhaseConfig::default()
        };
        let checker = PhaseChecker::new(config);
        
        let mut complex = create_complex_structure();
        
        // Initially, no phases are completed
        assert!(!complex.phase_completed);
        
        // Transfer phases
        checker.transfer_phase(&mut complex);
        
        // Now the phase should be completed
        assert!(complex.phase_completed);
        
        // Check that embedded phases are also completed
        if let Some((_, right)) = &complex.children {
            if let Some((_, v_vp)) = &right.children {
                if let Some((_, v_node)) = &v_vp.children {
                    assert!(v_node.phase_completed);
                } else {
                    panic!("Expected v node");
                }
            } else {
                panic!("Expected vP node");
            }
        } else {
            panic!("Expected CP children");
        }
    }
}