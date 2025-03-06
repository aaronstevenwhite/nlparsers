//! Constraints in Lexical-Functional Grammar
//!
//! This module provides a constraint solver for LFG that ensures
//! well-formedness conditions are satisfied.

use std::collections::HashMap;
use crate::lfg::c_structure::CNode;
use crate::lfg::f_structure::{FStructure, FValue, FConstraint};

/// A constraint solver for LFG
#[derive(Debug)]
pub struct Constraint {
    /// The C-structure node
    pub node: CNode,
    /// Map of node indices to F-structures
    pub f_structures: HashMap<usize, FStructure>,
    /// Next available F-structure ID
    pub next_id: usize,
}

impl Constraint {
    /// Create a new constraint solver for a syntax tree
    pub fn new(node: CNode) -> Self {
        Self {
            node,
            f_structures: HashMap::new(),
            next_id: 0,
        }
    }
    
    /// Generate a unique ID for an F-structure
    pub fn generate_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    /// Solve constraints and generate F-structures
    pub fn solve(&mut self) -> Option<CNode> {
        // Create a copy of the node that we'll update with F-structures
        let mut result_node = self.node.clone();
        
        // Process the constraints bottom-up
        self.process_constraints(&mut result_node)?;
        
        // Verify well-formedness conditions
        if self.check_well_formedness(&result_node) {
            Some(result_node)
        } else {
            None
        }
    }
    
    /// Process constraints on a node and its children
    fn process_constraints(&mut self, node: &mut CNode) -> Option<FStructure> {
        // First, process all children
        let mut child_f_structures = Vec::new();
        
        for child in &mut node.children {
            if let Some(fs) = self.process_constraints(child) {
                child_f_structures.push(fs);
            } else {
                return None; // Constraint solving failed
            }
        }
        
        // Now, create an F-structure for this node
        let id = self.generate_id();
        let mut f_structure = FStructure::new(id);
        
        // For lexical items, use the category's F-structure annotations
        if node.is_leaf() {
            // Apply lexical constraints (from the word's lexical entry)
            for constraint in &node.category.f_equations {
                // Create a clone to avoid borrowing f_structure both mutably and immutably
                let current_clone = f_structure.clone();
                self.apply_constraint(&mut f_structure, constraint, &current_clone)?;
            }
        } else {
            // For phrasal nodes, establish links with children based on rule annotations
            // In a real implementation, we'd look up the rule that created this node
            // and apply its annotations
            
            // For now, just link children based on typical LFG patterns
            if node.category.name == "S" && node.children.len() == 2 {
                // Typical S -> NP VP rule
                // First child (NP) is subject, second (VP) shares features with parent
                if let Some(subj_fs) = &child_f_structures.get(0) {
                    f_structure.set("SUBJ", FValue::Structure(Box::new((*subj_fs).clone())));
                }
                
                if let Some(vp_fs) = &child_f_structures.get(1) {
                    // Merge VP's attributes into S
                    for (attr, val) in &vp_fs.attributes {
                        if attr != "SUBJ" { // Avoid overwriting subject
                            f_structure.set(attr, val.clone());
                        }
                    }
                }
            } else if node.category.name == "VP" && node.children.len() == 2 {
                // Typical VP -> V NP rule
                // First child (V) shares features with parent, second (NP) is object
                if let Some(v_fs) = &child_f_structures.get(0) {
                    // Merge V's attributes into VP
                    for (attr, val) in &v_fs.attributes {
                        f_structure.set(attr, val.clone());
                    }
                }
                
                if let Some(obj_fs) = &child_f_structures.get(1) {
                    f_structure.set("OBJ", FValue::Structure(Box::new((*obj_fs).clone())));
                }
            } else if node.category.name == "NP" && node.children.len() == 2 {
                // Typical NP -> Det N rule
                // First child (Det) becomes DET attribute, second (N) shares features
                if let Some(det_fs) = &child_f_structures.get(0) {
                    f_structure.set("DET", FValue::Structure(Box::new((*det_fs).clone())));
                }
                
                if let Some(n_fs) = &child_f_structures.get(1) {
                    // Merge N's attributes into NP
                    for (attr, val) in &n_fs.attributes {
                        if attr != "DET" { // Avoid overwriting determiner
                            f_structure.set(attr, val.clone());
                        }
                    }
                }
            }
        }
        
        // Store and return the F-structure
        node.f_structure = Some(f_structure.clone());
        self.f_structures.insert(id, f_structure.clone());
        
        Some(f_structure)
    }
    
    /// Apply a specific constraint to an F-structure
    fn apply_constraint(
        &self,
        f_structure: &mut FStructure,
        constraint: &FConstraint,
        current: &FStructure,
    ) -> Option<()> {
        match constraint {
            FConstraint::Equation(lhs, rhs) => {
                // Simplified implementation of equation solving
                if lhs == "↑" && rhs == "↓" {
                    // ↑=↓ means parent and child share all features
                    // This is handled specially in process_constraints
                    return Some(());
                } else if lhs.starts_with("↑") && rhs == "↓" {
                    // ↑ATTR=↓ means child is value of parent's ATTR
                    // This is handled specially in process_constraints
                    return Some(());
                } else if !lhs.contains("↑") && !rhs.contains("↑") && !rhs.contains("↓") {
                    // Simple attribute = value case
                    f_structure.set(lhs, FValue::Atomic(rhs.clone()));
                }
                
                Some(())
            },
            FConstraint::ConstrainingEquation(lhs, rhs) => {
                // Ensure the value at lhs equals rhs
                if let Some(val) = f_structure.get(lhs) {
                    match val {
                        FValue::Atomic(s) if s == rhs => Some(()),
                        _ => None, // Constraint failed
                    }
                } else {
                    // If attribute doesn't exist, add it
                    f_structure.set(lhs, FValue::Atomic(rhs.clone()));
                    Some(())
                }
            },
            FConstraint::Containment(lhs, rhs) => {
                // Add to a set
                if let Some(FValue::Set(set)) = f_structure.get_mut(lhs) {
                    set.push(FValue::Atomic(rhs.clone()));
                } else {
                    f_structure.add_to_set(lhs, FValue::Atomic(rhs.clone()));
                }
                Some(())
            },
            FConstraint::Negation(path) => {
                // Ensure path doesn't exist
                if f_structure.has_attribute(path) {
                    None // Constraint failed
                } else {
                    Some(())
                }
            },
            FConstraint::Disjunction(left, right) => {
                // Try left constraint
                let mut fs_copy = f_structure.clone();
                if self.apply_constraint(&mut fs_copy, left, current).is_some() {
                    *f_structure = fs_copy;
                    return Some(());
                }
                
                // If left fails, try right
                self.apply_constraint(f_structure, right, current)
            },
            FConstraint::DefiningEquation(lhs, rhs) => {
                // Similar to Equation but with different semantics in LFG
                self.apply_constraint(f_structure, &FConstraint::Equation(lhs.clone(), rhs.clone()), current)
            },
            FConstraint::FunctionalUncertainty(lhs, rhs) => {
                // Handle functional uncertainty
                if lhs.contains('*') {
                    // Path with wildcards
                    if rhs == "↓" {
                        // Create a copy of the current node's f-structure
                        let value = FValue::Structure(Box::new(current.clone()));
                        if f_structure.apply_functional_uncertainty(lhs, value) {
                            Some(())
                        } else {
                            None
                        }
                    } else {
                        // Other value
                        if f_structure.apply_functional_uncertainty(lhs, FValue::Atomic(rhs.clone())) {
                            Some(())
                        } else {
                            None
                        }
                    }
                } else {
                    // Regular path, use normal equation
                    self.apply_constraint(f_structure, &FConstraint::Equation(lhs.clone(), rhs.clone()), current)
                }
            },
            FConstraint::OffPathConstraint(lhs, rhs) => {
                // Check if two paths have the same value
                if f_structure.apply_off_path_constraint(lhs, rhs) {
                    Some(())
                } else {
                    None
                }
            },
            FConstraint::InsideOut(_lhs, _rhs) => {
                // Inside-out functional application (simplified)
                // In a full implementation, this would require more complex processing
                // to handle inside-out functional uncertainty
                Some(())
            },
            FConstraint::SetMembership(lhs, rhs) => {
                // Similar to containment but with different semantics
                self.apply_constraint(f_structure, &FConstraint::Containment(lhs.clone(), rhs.clone()), current)
            },
        }
    }
    
    /// Check if all well-formedness conditions are satisfied
    fn check_well_formedness(&self, node: &CNode) -> bool {
        // Enhanced well-formedness checking
        if let Some(fs) = &node.f_structure {
            return fs.is_well_formed();
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lfg::c_structure::Category;
    
    // Helper to create a simple syntax tree
    fn create_test_tree() -> CNode {
        // Create categories
        let s = Category::s();
        let np = Category::np();
        let vp = Category::vp();
        let v = Category::new("V");
        let n = Category::new("N");
        let det = Category::new("Det");
        
        // Create lexical nodes
        let the = CNode::leaf("the", det);
        let cat = CNode::leaf("cat", n);
        let sleeps = CNode::leaf("sleeps", v);
        
        // Create NP
        let np_node = CNode::internal(np, vec![the, cat], "NP_rule");
        
        // Create VP
        let vp_node = CNode::internal(vp, vec![sleeps], "VP_rule");
        
        // Create S
        CNode::internal(s, vec![np_node, vp_node], "S_rule")
    }
    
    #[test]
    fn test_constraint_solver() {
        let tree = create_test_tree();
        let mut solver = Constraint::new(tree);
        
        let result = solver.solve();
        assert!(result.is_some());
        
        let solved_tree = result.unwrap();
        
        // Check that all nodes have F-structures
        assert!(solved_tree.f_structure.is_some());
        assert!(solved_tree.children[0].f_structure.is_some()); // NP
        assert!(solved_tree.children[1].f_structure.is_some()); // VP
    }
    
    #[test]
    fn test_process_constraints() {
        let tree = create_test_tree();
        let mut solver = Constraint::new(tree);
        
        let mut tree_copy = solver.node.clone();
        let fs = solver.process_constraints(&mut tree_copy);
        
        assert!(fs.is_some());
        
        let f_structure = fs.unwrap();
        
        // Check that SUBJ attribute was created
        assert!(f_structure.has_attribute("SUBJ"));
    }
    
    #[test]
    fn test_apply_constraint() {
        let mut fs = FStructure::new(1);
        let solver = Constraint::new(create_test_tree());
        
        // Test simple equation
        let constraint = FConstraint::Equation("NUM".to_string(), "sg".to_string());
        let current = fs.clone();
        let result = solver.apply_constraint(&mut fs, &constraint, &current);
        
        assert!(result.is_some());
        assert_eq!(fs.get("NUM"), Some(&FValue::Atomic("sg".to_string())));
        
        // Test containment
        let constraint = FConstraint::Containment("ADJUNCTS".to_string(), "ADJ1".to_string());
        let current = fs.clone();
        let result = solver.apply_constraint(&mut fs, &constraint, &current);
        
        assert!(result.is_some());
        if let Some(FValue::Set(set)) = fs.get("ADJUNCTS") {
            assert_eq!(set.len(), 1);
        } else {
            panic!("Expected set value");
        }
        
        // Test constraining equation (success)
        let constraint = FConstraint::ConstrainingEquation("NUM".to_string(), "sg".to_string());
        let current = fs.clone();
        let result = solver.apply_constraint(&mut fs, &constraint, &current);
        
        assert!(result.is_some());
        
        // Test constraining equation (failure)
        let constraint = FConstraint::ConstrainingEquation("NUM".to_string(), "pl".to_string());
        let current = fs.clone();
        let result = solver.apply_constraint(&mut fs, &constraint, &current);
        
        assert!(result.is_none());
    }
}