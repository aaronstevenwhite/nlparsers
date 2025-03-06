//! Mapping between F-structure and R-structure in LRFG
//!
//! This module provides the mapping rules that relate F-structure features
//! to R-structure features.

use std::fmt;
use std::collections::HashMap;
use crate::lfg::f_structure::{FStructure, FValue};
use crate::lrfg::r_structure::{RStructure, RNode};

/// A mapping rule between F-structure and R-structure
#[derive(Debug, Clone)]
pub struct MappingRule {
    /// F-structure path (e.g., "SUBJ NUM")
    pub f_path: String,
    /// F-structure value to match
    pub f_value: Option<String>,
    /// R-structure feature name
    pub r_name: String,
    /// R-structure feature value
    pub r_value: String,
}

impl MappingRule {
    /// Create a new mapping rule
    pub fn new(f_path: &str, r_name: &str, r_value: &str) -> Self {
        Self {
            f_path: f_path.to_string(),
            f_value: None,
            r_name: r_name.to_string(),
            r_value: r_value.to_string(),
        }
    }
    
    /// Create a new mapping rule with a specific F-structure value
    pub fn with_value(f_path: &str, f_value: &str, r_name: &str, r_value: &str) -> Self {
        Self {
            f_path: f_path.to_string(),
            f_value: Some(f_value.to_string()),
            r_name: r_name.to_string(),
            r_value: r_value.to_string(),
        }
    }
    
    /// Check if this rule applies to an F-structure
    pub fn applies_to(&self, f_structure: &FStructure) -> bool {
        // Resolve the path in the F-structure
        let parts: Vec<&str> = self.f_path.split_whitespace().collect();
        let mut current = f_structure;
        
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - check value
                if let Some(value) = current.get(part) {
                    if let Some(expected) = &self.f_value {
                        match value {
                            FValue::Atomic(s) => return s == expected,
                            _ => return false,
                        }
                    }
                    return true; // No specific value to match
                }
                return false;
            } else {
                // Navigate to embedded structure
                if let Some(FValue::Structure(fs)) = current.get(part) {
                    current = fs;
                } else {
                    return false;
                }
            }
        }
        
        false
    }
}

impl fmt::Display for MappingRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.f_value {
            write!(f, "{} = {} → {}:{}", self.f_path, value, self.r_name, self.r_value)
        } else {
            write!(f, "{} → {}:{}", self.f_path, self.r_name, self.r_value)
        }
    }
}

/// Mapping between F-structure and R-structure
#[derive(Debug, Clone)]
pub struct FRMapping {
    /// Mapping rules
    pub rules: Vec<MappingRule>,
}

impl FRMapping {
    /// Create a new empty mapping
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
    
    /// Add a mapping rule
    pub fn add_rule(&mut self, rule: MappingRule) {
        self.rules.push(rule);
    }
    
    /// Apply mapping rules to create an R-structure from an F-structure
    pub fn apply(&self, f_structure: &FStructure) -> RStructure {
        let mut r_structure = RStructure::new();
        
        // Apply rules to root node
        self.apply_to_node(f_structure, &mut r_structure.root);
        
        // Process embedded structures
        for (attr, value) in &f_structure.attributes {
            if let FValue::Structure(fs) = value {
                if ["SUBJ", "OBJ", "COMP", "ADJUNCT"].contains(&attr.as_str()) {
                    let node_id = r_structure.new_node();
                    let mut node = RNode::new(node_id);
                    
                    // Add grammatical function as a feature
                    node.add_feature("gf", attr);
                    
                    // Apply rules to this node
                    self.apply_to_node(fs, &mut node);
                    
                    // Add as child of root
                    r_structure.root.add_child(node);
                }
            }
        }
        
        r_structure
    }
    
    /// Apply mapping rules to a specific node
    fn apply_to_node(&self, f_structure: &FStructure, r_node: &mut RNode) {
        // Apply all matching rules
        for rule in &self.rules {
            if rule.applies_to(f_structure) {
                r_node.add_feature(&rule.r_name, &rule.r_value);
            }
        }
        
        // Special handling for PRED
        if let Some(FValue::Semantic(pred, _)) = f_structure.get("PRED") {
            r_node.add_feature("pred", pred);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lfg::f_structure::FStructure;
    
    #[test]
    fn test_mapping_rule() {
        let rule = MappingRule::with_value("NUM", "sg", "num", "sg");
        
        let mut fs = FStructure::new(0);
        fs.set("NUM", FValue::Atomic("sg".to_string()));
        
        assert!(rule.applies_to(&fs));
        
        // Test with non-matching value
        let mut fs2 = FStructure::new(0);
        fs2.set("NUM", FValue::Atomic("pl".to_string()));
        
        assert!(!rule.applies_to(&fs2));
    }
    
    #[test]
    fn test_fr_mapping() {
        let mut mapping = FRMapping::new();
        
        // Add rules
        mapping.add_rule(MappingRule::with_value("NUM", "sg", "num", "sg"));
        mapping.add_rule(MappingRule::with_value("PERS", "3", "pers", "3"));
        mapping.add_rule(MappingRule::with_value("TENSE", "pres", "tense", "pres"));
        
        // Create an F-structure
        let mut fs = FStructure::new(0);
        fs.set("NUM", FValue::Atomic("sg".to_string()));
        fs.set("PERS", FValue::Atomic("3".to_string()));
        fs.set("TENSE", FValue::Atomic("pres".to_string()));
        fs.set_pred("cat", vec![]);
        
        // Apply mapping
        let r_structure = mapping.apply(&fs);
        
        // Check that features were mapped correctly
        assert!(r_structure.root.has_feature("num", "sg"));
        assert!(r_structure.root.has_feature("pers", "3"));
        assert!(r_structure.root.has_feature("tense", "pres"));
        assert!(r_structure.root.has_feature("pred", "cat"));
    }
} 