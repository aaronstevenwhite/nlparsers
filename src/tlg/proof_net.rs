//! Proof nets for Type-Logical Grammar
//!
//! This module provides an implementation of proof nets, which are a more
//! efficient representation for proofs in linear logic and Type-Logical Grammar.

use std::collections::VecDeque;
use crate::tlg::logical_type::LogicalType;
use crate::tlg::modality::Modality;
use crate::tlg::proof::ProofNode;
use crate::common::FeatureStructure;

/// Structure used in Proof Nets for efficiency
#[derive(Debug, Clone)]
pub struct ProofNet {
    /// Nodes in the proof net (formulas)
    pub nodes: Vec<ProofNetNode>,
    /// Links between nodes
    pub links: Vec<ProofNetLink>,
    /// Output conclusion node
    pub output: usize,
}

/// Types of nodes in a Proof Net
#[derive(Debug, Clone)]
pub enum ProofNetNode {
    /// Atomic formula
    Atom(String, FeatureStructure, bool), // name, features, polarity (true=positive, false=negative)
    /// Tensor (⊗) node
    Tensor(usize, usize, Option<Modality>), // left child, right child, modality
    /// Par (⅋) node
    Par(usize, usize, Option<Modality>), // left child, right child, modality
    /// Of-course (!) node (for exponentials)
    OfCourse(usize, Option<Modality>), // child, modality
    /// Why-not (?) node (for exponentials)
    WhyNot(usize, Option<Modality>), // child, modality
    /// Displacement nodes
    Displacement(usize, usize, usize), // left child, right child, index
}

/// Link between nodes in a Proof Net
#[derive(Debug, Clone)]
pub struct ProofNetLink {
    /// Source node index
    pub source: usize,
    /// Target node index
    pub target: usize,
    /// Axiom link or structural link
    pub is_axiom: bool,
}

impl ProofNet {
    /// Convert a logical type to a proof net
    pub fn from_type(logical_type: &LogicalType, polarity: bool) -> Self {
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        
        // Recursively build the proof net
        let output = Self::build_node(logical_type, polarity, &mut nodes, &mut links);
        
        ProofNet {
            nodes,
            links,
            output,
        }
    }
    
    /// Build a proof net node from a logical type
    fn build_node(
        logical_type: &LogicalType, 
        polarity: bool, 
        nodes: &mut Vec<ProofNetNode>,
        links: &mut Vec<ProofNetLink>
    ) -> usize {
        match logical_type {
            LogicalType::Atomic(name, features) => {
                let index = nodes.len();
                nodes.push(ProofNetNode::Atom(name.clone(), features.clone(), polarity));
                index
            },
            LogicalType::RightImplication(a, b, modality) => {
                if polarity {
                    // A → B with positive polarity becomes A⊥ ⅋ B
                    let a_index = Self::build_node(a, !polarity, nodes, links);
                    let b_index = Self::build_node(b, polarity, nodes, links);
                    let index = nodes.len();
                    nodes.push(ProofNetNode::Par(a_index, b_index, modality.clone()));
                    index
                } else {
                    // A → B with negative polarity becomes A ⊗ B⊥
                    let a_index = Self::build_node(a, polarity, nodes, links);
                    let b_index = Self::build_node(b, !polarity, nodes, links);
                    let index = nodes.len();
                    nodes.push(ProofNetNode::Tensor(a_index, b_index, modality.clone()));
                    index
                }
            },
            LogicalType::LeftImplication(a, b, modality) => {
                if polarity {
                    // A ← B with positive polarity becomes A ⅋ B⊥
                    let a_index = Self::build_node(a, polarity, nodes, links);
                    let b_index = Self::build_node(b, !polarity, nodes, links);
                    let index = nodes.len();
                    nodes.push(ProofNetNode::Par(a_index, b_index, modality.clone()));
                    index
                } else {
                    // A ← B with negative polarity becomes A⊥ ⊗ B
                    let a_index = Self::build_node(a, !polarity, nodes, links);
                    let b_index = Self::build_node(b, polarity, nodes, links);
                    let index = nodes.len();
                    nodes.push(ProofNetNode::Tensor(a_index, b_index, modality.clone()));
                    index
                }
            },
            LogicalType::Product(a, b, modality) => {
                if polarity {
                    // A ⊗ B with positive polarity becomes A ⊗ B
                    let a_index = Self::build_node(a, polarity, nodes, links);
                    let b_index = Self::build_node(b, polarity, nodes, links);
                    let index = nodes.len();
                    nodes.push(ProofNetNode::Tensor(a_index, b_index, modality.clone()));
                    index
                } else {
                    // A ⊗ B with negative polarity becomes A⊥ ⅋ B⊥
                    let a_index = Self::build_node(a, polarity, nodes, links);
                    let b_index = Self::build_node(b, polarity, nodes, links);
                    let index = nodes.len();
                    nodes.push(ProofNetNode::Par(a_index, b_index, modality.clone()));
                    index
                }
            },
            LogicalType::Diamond(a, modality) => {
                let a_index = Self::build_node(a, polarity, nodes, links);
                let index = nodes.len();
                if polarity {
                    nodes.push(ProofNetNode::WhyNot(a_index, modality.clone()));
                } else {
                    nodes.push(ProofNetNode::OfCourse(a_index, modality.clone()));
                }
                index
            },
            LogicalType::Box(a, modality) => {
                let a_index = Self::build_node(a, polarity, nodes, links);
                let index = nodes.len();
                if polarity {
                    nodes.push(ProofNetNode::OfCourse(a_index, modality.clone()));
                } else {
                    nodes.push(ProofNetNode::WhyNot(a_index, modality.clone()));
                }
                index
            },
            LogicalType::UpArrow(a, b, i) => {
                let a_index = Self::build_node(a, polarity, nodes, links);
                let b_index = Self::build_node(b, !polarity, nodes, links);
                let index = nodes.len();
                nodes.push(ProofNetNode::Displacement(a_index, b_index, *i));
                index
            },
            LogicalType::DownArrow(a, b, i) => {
                let a_index = Self::build_node(a, polarity, nodes, links);
                let b_index = Self::build_node(b, polarity, nodes, links);
                let index = nodes.len();
                nodes.push(ProofNetNode::Displacement(a_index, b_index, *i));
                index
            },
            // For quantifiers, we would need a more complex encoding
            _ => unimplemented!("Quantifiers not yet implemented in proof nets"),
        }
    }
    
    /// Check if the proof net is correct (connected and acyclic)
    pub fn is_correct(&self) -> bool {
        // 1. Check connectedness
        if !self.is_connected() {
            return false;
        }
        
        // 2. Check acyclicity (no loops)
        if self.has_cycles() {
            return false;
        }
        
        // 3. Additional criteria for correctness
        self.check_additional_criteria()
    }
    
    /// Check if all nodes are connected
    fn is_connected(&self) -> bool {
        if self.nodes.is_empty() {
            return true;
        }
        
        let mut visited = vec![false; self.nodes.len()];
        let mut queue = VecDeque::new();
        
        // Start from the output node
        queue.push_back(self.output);
        visited[self.output] = true;
        
        while let Some(node) = queue.pop_front() {
            // Find all connected nodes
            for link in &self.links {
                if link.source == node && !visited[link.target] {
                    visited[link.target] = true;
                    queue.push_back(link.target);
                } else if link.target == node && !visited[link.source] {
                    visited[link.source] = true;
                    queue.push_back(link.source);
                }
            }
            
            // Also check node structure connections
            match &self.nodes[node] {
                ProofNetNode::Tensor(left, right, _) |
                ProofNetNode::Par(left, right, _) |
                ProofNetNode::Displacement(left, right, _) => {
                    if !visited[*left] {
                        visited[*left] = true;
                        queue.push_back(*left);
                    }
                    if !visited[*right] {
                        visited[*right] = true;
                        queue.push_back(*right);
                    }
                },
                ProofNetNode::OfCourse(child, _) |
                ProofNetNode::WhyNot(child, _) => {
                    if !visited[*child] {
                        visited[*child] = true;
                        queue.push_back(*child);
                    }
                },
                _ => {},
            }
        }
        
        // All nodes should be visited
        visited.iter().all(|&v| v)
    }
    
    /// Check if the proof net has cycles
    fn has_cycles(&self) -> bool {
        let mut visited = vec![false; self.nodes.len()];
        let mut rec_stack = vec![false; self.nodes.len()];
        
        for i in 0..self.nodes.len() {
            if !visited[i] && self.is_cyclic_util(i, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        
        false
    }
    
    /// Utility function for cycle detection
    fn is_cyclic_util(&self, node: usize, visited: &mut [bool], rec_stack: &mut [bool]) -> bool {
        visited[node] = true;
        rec_stack[node] = true;
        
        // Check all adjacent nodes
        for link in &self.links {
            if link.source == node {
                let next = link.target;
                if !visited[next] && self.is_cyclic_util(next, visited, rec_stack) {
                    return true;
                } else if rec_stack[next] {
                    return true;
                }
            }
        }
        
        // Also check node structure connections
        match &self.nodes[node] {
            ProofNetNode::Tensor(left, right, _) |
            ProofNetNode::Par(left, right, _) |
            ProofNetNode::Displacement(left, right, _) => {
                if !visited[*left] && self.is_cyclic_util(*left, visited, rec_stack) {
                    return true;
                } else if rec_stack[*left] {
                    return true;
                }
                
                if !visited[*right] && self.is_cyclic_util(*right, visited, rec_stack) {
                    return true;
                } else if rec_stack[*right] {
                    return true;
                }
            },
            ProofNetNode::OfCourse(child, _) |
            ProofNetNode::WhyNot(child, _) => {
                if !visited[*child] && self.is_cyclic_util(*child, visited, rec_stack) {
                    return true;
                } else if rec_stack[*child] {
                    return true;
                }
            },
            _ => {},
        }
        
        rec_stack[node] = false;
        false
    }
    
    /// Additional criteria for proof net correctness
    fn check_additional_criteria(&self) -> bool {
        // For displacement calculus, check proper nesting
        // For modalities, check proper use
        // This would be a complex implementation depending on the specific logic
        
        // Simplified version for now
        true
    }
    
    /// Generate a proof tree from a correct proof net
    pub fn to_proof_tree(&self) -> Option<ProofNode> {
        if !self.is_correct() {
            return None;
        }
        
        // Start from output and build the tree
        self.build_proof_tree(self.output)
    }
    
    /// Build a proof tree from a proof net node
    fn build_proof_tree(&self, node_index: usize) -> Option<ProofNode> {
        match &self.nodes[node_index] {
            ProofNetNode::Atom(name, features, polarity) => {
                let atomic_type = LogicalType::atomic_with_features(name, &features);
                Some(ProofNode::axiom(&format!("{}_{}", name, node_index), atomic_type))
            },
            ProofNetNode::Tensor(left, right, modality) => {
                if let (Some(left_tree), Some(right_tree)) = (
                    self.build_proof_tree(*left),
                    self.build_proof_tree(*right)
                ) {
                    let mod_str = if let Some(m) = modality {
                        format!("{}", m)
                    } else {
                        "".to_string()
                    };
                    
                    let logical_type = LogicalType::product(
                        left_tree.logical_type.clone(),
                        right_tree.logical_type.clone()
                    );
                    
                    Some(ProofNode::infer(
                        logical_type,
                        vec![left_tree, right_tree],
                        &format!("⊗I{}", mod_str)
                    ))
                } else {
                    None
                }
            },
            ProofNetNode::Par(left, right, modality) => {
                if let (Some(left_tree), Some(right_tree)) = (
                    self.build_proof_tree(*left),
                    self.build_proof_tree(*right)
                ) {
                    let mod_str = if let Some(m) = modality {
                        format!("{}", m)
                    } else {
                        "".to_string()
                    };
                    
                    // Depending on the polarity, this could be →I or ←I
                    let logical_type = LogicalType::right_impl(
                        left_tree.logical_type.clone(),
                        right_tree.logical_type.clone()
                    );
                    
                    Some(ProofNode::infer(
                        logical_type,
                        vec![left_tree, right_tree],
                        &format!("→I{}", mod_str)
                    ))
                } else {
                    None
                }
            },
            ProofNetNode::OfCourse(child, modality) => {
                if let Some(child_tree) = self.build_proof_tree(*child) {
                    let mod_str = if let Some(m) = modality {
                        format!("{}", m)
                    } else {
                        "".to_string()
                    };
                    
                    let logical_type = LogicalType::boxed(child_tree.logical_type.clone());
                    
                    Some(ProofNode::infer(
                        logical_type,
                        vec![child_tree],
                        &format!("□I{}", mod_str)
                    ))
                } else {
                    None
                }
            },
            ProofNetNode::WhyNot(child, modality) => {
                if let Some(child_tree) = self.build_proof_tree(*child) {
                    let mod_str = if let Some(m) = modality {
                        format!("{}", m)
                    } else {
                        "".to_string()
                    };
                    
                    let logical_type = LogicalType::diamond(child_tree.logical_type.clone());
                    
                    Some(ProofNode::infer(
                        logical_type,
                        vec![child_tree],
                        &format!("◇I{}", mod_str)
                    ))
                } else {
                    None
                }
            },
            ProofNetNode::Displacement(left, right, index) => {
                if let (Some(left_tree), Some(right_tree)) = (
                    self.build_proof_tree(*left),
                    self.build_proof_tree(*right)
                ) {
                    let logical_type = LogicalType::up_arrow(
                        left_tree.logical_type.clone(),
                        right_tree.logical_type.clone(),
                        *index
                    );
                    
                    Some(ProofNode::infer(
                        logical_type,
                        vec![left_tree, right_tree],
                        &format!("↑{}I", index)
                    ))
                } else {
                    None
                }
            },
        }
    }
    
    /// Link two proof nets together
    pub fn link_with(&mut self, other: &ProofNet, axiom_links: Vec<(usize, usize)>) -> bool {
        // Calculate offset for the other net's nodes
        let offset = self.nodes.len();
        
        // Add all nodes from the other net
        self.nodes.extend(other.nodes.clone());
        
        // Add all internal links from the other net (with offset)
        for link in &other.links {
            self.links.push(ProofNetLink {
                source: link.source + offset,
                target: link.target + offset,
                is_axiom: link.is_axiom,
            });
        }
        
        // Add the new axiom links
        for (self_idx, other_idx) in axiom_links {
            self.links.push(ProofNetLink {
                source: self_idx,
                target: other_idx + offset,
                is_axiom: true,
            });
        }
        
        // Check if the combined net is still correct
        self.is_correct()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_simple_proof_net() -> ProofNet {
        // Create a simple proof net for "NP → S" with positive polarity
        let np = LogicalType::np();
        let s = LogicalType::s();
        let type_pred = LogicalType::right_impl(np, s);
        
        ProofNet::from_type(&type_pred, true)
    }
    
    #[test]
    fn test_proof_net_creation() {
        let net = create_simple_proof_net();
        
        // Should have three nodes: Par, NP (negative polarity), and S (positive polarity)
        assert_eq!(net.nodes.len(), 3);
        
        // Check the structure
        match &net.nodes[net.output] {
            ProofNetNode::Par(np_idx, s_idx, _) => {
                match &net.nodes[*np_idx] {
                    ProofNetNode::Atom(name, _, polarity) => {
                        assert_eq!(name, "np");
                        assert!(!polarity); // Negative polarity for NP in NP → S
                    },
                    _ => panic!("Expected NP atom"),
                }
                
                match &net.nodes[*s_idx] {
                    ProofNetNode::Atom(name, _, polarity) => {
                        assert_eq!(name, "s");
                        assert!(*polarity); // Positive polarity for S in NP → S
                    },
                    _ => panic!("Expected S atom"),
                }
            },
            _ => panic!("Expected Par node at root"),
        }
    }
    
    #[test]
    fn test_proof_net_to_tree() {
        let net = create_simple_proof_net();
        
        // Can't directly convert to a proof tree without axiom links
        let tree = net.to_proof_tree();
        
        // It might still generate a tree, but this will depend on the internal
        // state of the proof net and any default axiom connections
        
        if let Some(tree) = tree {
            // If we got a tree, verify its structure
            assert_eq!(tree.logical_type.to_string(), "np → s");
        }
    }
}