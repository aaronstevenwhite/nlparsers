//! Constituent structure (C-structure) for Lexical-Functional Grammar
//!
//! C-structure in LFG is represented as a phrase structure tree,
//! similar to traditional X-bar theory or context-free grammar.

use std::fmt;
use crate::common::{FeatureStructure, FeatureValue};
use crate::lfg::f_structure::{FStructure, FConstraint};
use crate::common::ParseNode;

/// A syntactic category in LFG C-structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Category {
    /// Name of the category (e.g., NP, VP, S)
    pub name: String,
    /// Morphosyntactic features (person, number, etc.)
    pub features: FeatureStructure,
    /// Mapping to F-structure
    pub f_equations: Vec<FConstraint>,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        
        if !self.features.features.is_empty() {
            write!(f, "{}", self.features)?;
        }
        
        if !self.f_equations.is_empty() {
            write!(f, " ")?;
            for (i, eq) in self.f_equations.iter().enumerate() {
                if i > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{}", eq)?;
            }
        }
        
        Ok(())
    }
}

impl Category {
    /// Create a new category with just a name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            features: FeatureStructure::new(),
            f_equations: Vec::new(),
        }
    }
    
    /// Create a new category with features
    pub fn with_features(name: &str, features: FeatureStructure) -> Self {
        Self {
            name: name.to_string(),
            features,
            f_equations: Vec::new(),
        }
    }
    
    /// Create a category with F-structure constraints
    pub fn with_constraints(name: &str, constraints: Vec<FConstraint>) -> Self {
        Self {
            name: name.to_string(),
            features: FeatureStructure::new(),
            f_equations: constraints,
        }
    }
    
    /// Create a category with both features and constraints
    pub fn with_features_and_constraints(
        name: &str, 
        features: FeatureStructure,
        constraints: Vec<FConstraint>
    ) -> Self {
        Self {
            name: name.to_string(),
            features,
            f_equations: constraints,
        }
    }
    
    /// Add an F-structure constraint
    pub fn add_constraint(&mut self, constraint: FConstraint) {
        self.f_equations.push(constraint);
    }
    
    /// Add multiple F-structure constraints
    pub fn add_constraints(&mut self, constraints: Vec<FConstraint>) {
        self.f_equations.extend(constraints);
    }
    
    /// Create common S category
    pub fn s() -> Self {
        Self::new("S")
    }
    
    /// Create common NP category
    pub fn np() -> Self {
        Self::new("NP")
    }
    
    /// Create common VP category
    pub fn vp() -> Self {
        Self::new("VP")
    }
    
    /// Create NP with case and number features
    pub fn np_with_features(case: &str, number: &str) -> Self {
        let mut features = FeatureStructure::new();
        features.add("case", FeatureValue::Atomic(case.to_string()));
        features.add("num", FeatureValue::Atomic(number.to_string()));
        Self::with_features("NP", features)
    }
    
    /// Unify this category with another
    pub fn unify(&self, other: &Category) -> Option<Category> {
        // Categories must have the same name to unify
        if self.name != other.name {
            return None;
        }
        
        // Unify features
        let unified_features = match self.features.unify(&other.features) {
            Some(unified) => unified,
            None => return None,
        };
        
        // Combine F-structure constraints
        let mut combined_equations = self.f_equations.clone();
        combined_equations.extend(other.f_equations.clone());
        
        Some(Category {
            name: self.name.clone(),
            features: unified_features,
            f_equations: combined_equations,
        })
    }
    
    /// Create a coordination category
    pub fn conj() -> Self {
        Self::new("CONJ")
    }
    
    /// Create a coordinated category (e.g., NP[+CONJ])
    pub fn coordinated(base: &str) -> Self {
        let mut cat = Self::new(base);
        let mut features = FeatureStructure::new();
        features.add("CONJ", FeatureValue::Atomic("yes".to_string()));
        cat.features = features;
        cat
    }
}

/// A node in the C-structure tree
#[derive(Debug, Clone)]
pub struct CNode {
    /// The syntactic category
    pub category: Category,
    /// The original word if this is a leaf
    pub word: Option<String>,
    /// Child nodes
    pub children: Vec<CNode>,
    /// The rule used to derive this node
    pub rule: Option<String>,
    /// Associated F-structure
    pub f_structure: Option<FStructure>,
}

impl CNode {
    /// Create a new leaf node
    pub fn leaf(word: &str, category: Category) -> Self {
        CNode {
            category,
            word: Some(word.to_string()),
            children: Vec::new(),
            rule: None,
            f_structure: None,
        }
    }
    
    /// Create a new internal node
    pub fn internal(category: Category, children: Vec<CNode>, rule: &str) -> Self {
        CNode {
            category,
            word: None,
            children,
            rule: Some(rule.to_string()),
            f_structure: None,
        }
    }
    
    /// Set the F-structure for this node
    pub fn with_f_structure(mut self, f_structure: FStructure) -> Self {
        self.f_structure = Some(f_structure);
        self
    }
    
    /// Check if this node is a leaf
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
    
    /// Get the depth of this tree
    pub fn depth(&self) -> usize {
        if self.is_leaf() {
            0
        } else {
            1 + self.children.iter().map(|child| child.depth()).max().unwrap_or(0)
        }
    }
    
    /// Create an empty category node
    pub fn empty(category: Category) -> Self {
        CNode {
            category,
            word: None,
            children: Vec::new(),
            rule: None,
            f_structure: None,
        }
    }
    
    /// Check if this node is an empty category
    pub fn is_empty_category(&self) -> bool {
        self.word.is_none() && self.children.is_empty()
    }
    
    /// Check if this node represents coordination
    pub fn is_coordination(&self) -> bool {
        self.category.name == "CONJ" || 
        (self.rule.is_some() && self.rule.as_ref().unwrap().contains("Coordination"))
    }
}

impl fmt::Display for CNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_tree(node: &CNode, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let indent_str = " ".repeat(indent);
            
            if let Some(word) = &node.word {
                writeln!(f, "{}{}[{}]", indent_str, word, node.category)?;
            } else if let Some(rule) = &node.rule {
                writeln!(f, "{}{}[{}]", indent_str, rule, node.category)?;
                for child in &node.children {
                    print_tree(child, indent + 2, f)?;
                }
            }
            
            Ok(())
        }
        
        print_tree(self, 0, f)
    }
}

impl ParseNode for CNode {
    type Cat = Category;
    
    fn category(&self) -> &Self::Cat {
        &self.category
    }
    
    fn word(&self) -> Option<&str> {
        self.word.as_deref()
    }
    
    fn children(&self) -> &[Self] {
        &self.children
    }
    
    fn rule(&self) -> Option<&str> {
        self.rule.as_deref()
    }
}

/// Complete C-structure tree with associated context
#[derive(Debug, Clone)]
pub struct CStructure {
    /// Root node of the tree
    pub root: CNode,
    /// The words of the sentence
    pub words: Vec<String>,
}

impl CStructure {
    /// Create a new C-structure
    pub fn new(root: CNode, words: Vec<String>) -> Self {
        Self { root, words }
    }
    
    /// Get all the leaf nodes
    pub fn leaves(&self) -> Vec<&CNode> {
        fn collect_leaves<'a>(node: &'a CNode, leaves: &mut Vec<&'a CNode>) {
            if node.is_leaf() {
                leaves.push(node);
            } else {
                for child in &node.children {
                    collect_leaves(child, leaves);
                }
            }
        }
        
        let mut leaves = Vec::new();
        collect_leaves(&self.root, &mut leaves);
        leaves
    }
    
    /// Get all the F-structures in this tree
    pub fn f_structures(&self) -> Vec<&FStructure> {
        fn collect_f_structures<'a>(node: &'a CNode, structures: &mut Vec<&'a FStructure>) {
            if let Some(fs) = &node.f_structure {
                structures.push(fs);
            }
            
            for child in &node.children {
                collect_f_structures(child, structures);
            }
        }
        
        let mut structures = Vec::new();
        collect_f_structures(&self.root, &mut structures);
        structures
    }
}

impl fmt::Display for CStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C-structure for: \"{}\"\n", self.words.join(" "))?;
        write!(f, "{}", self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lfg::f_structure::FConstraint;
    
    #[test]
    fn test_category_creation() {
        let s = Category::s();
        assert_eq!(s.name, "S");
        assert!(s.features.features.is_empty());
        assert!(s.f_equations.is_empty());
        
        let mut features = FeatureStructure::new();
        features.add("num", FeatureValue::Atomic("sg".to_string()));
        
        let np_sg = Category::with_features("NP", features);
        assert_eq!(np_sg.name, "NP");
        assert_eq!(np_sg.features.get("num"), Some(&FeatureValue::Atomic("sg".to_string())));
    }
    
    #[test]
    fn test_category_with_constraints() {
        let constraints = vec![
            FConstraint::Equation("↑SUBJ".to_string(), "↓".to_string()),
        ];
        
        let np_subj = Category::with_constraints("NP", constraints);
        assert_eq!(np_subj.name, "NP");
        assert_eq!(np_subj.f_equations.len(), 1);
        
        // Test display with constraints
        assert_eq!(np_subj.to_string(), "NP ↑SUBJ=↓");
    }
    
    #[test]
    fn test_category_unification() {
        // Create two categories with different features
        let mut feat1 = FeatureStructure::new();
        feat1.add("num", FeatureValue::Atomic("sg".to_string()));
        
        let mut feat2 = FeatureStructure::new();
        feat2.add("per", FeatureValue::Atomic("3".to_string()));
        
        let cat1 = Category::with_features("NP", feat1);
        let cat2 = Category::with_features("NP", feat2);
        
        // They should unify
        let unified = cat1.unify(&cat2);
        assert!(unified.is_some());
        
        let unified_cat = unified.unwrap();
        assert_eq!(unified_cat.name, "NP");
        assert_eq!(unified_cat.features.get("num"), Some(&FeatureValue::Atomic("sg".to_string())));
        assert_eq!(unified_cat.features.get("per"), Some(&FeatureValue::Atomic("3".to_string())));
        
        // Different categories should not unify
        let s = Category::s();
        let unified_fail = cat1.unify(&s);
        assert!(unified_fail.is_none());
    }
    
    #[test]
    fn test_cnode_creation() {
        let np = Category::np();
        let john = CNode::leaf("John", np.clone());
        
        assert_eq!(john.word, Some("John".to_string()));
        assert!(john.is_leaf());
        assert_eq!(john.depth(), 0);
        
        // Test internal node
        let vp = Category::vp();
        let s = Category::s();
        
        let vp_node = CNode::internal(vp, vec![], "VP_rule");
        let s_node = CNode::internal(s, vec![john, vp_node], "S_rule");
        
        assert_eq!(s_node.depth(), 1);
        assert_eq!(s_node.children.len(), 2);
        assert_eq!(s_node.rule, Some("S_rule".to_string()));
    }
    
    #[test]
    fn test_cstructure() {
        let np = Category::np();
        let vp = Category::vp();
        let s = Category::s();
        
        let john = CNode::leaf("John", np.clone());
        let sleeps = CNode::leaf("sleeps", vp.clone());
        
        let vp_node = CNode::internal(vp, vec![sleeps], "VP_rule");
        let s_node = CNode::internal(s, vec![john, vp_node], "S_rule");
        
        let words = vec!["John".to_string(), "sleeps".to_string()];
        let c_structure = CStructure::new(s_node, words);
        
        let leaves = c_structure.leaves();
        assert_eq!(leaves.len(), 2);
        assert_eq!(leaves[0].word, Some("John".to_string()));
        assert_eq!(leaves[1].word, Some("sleeps".to_string()));
    }
}