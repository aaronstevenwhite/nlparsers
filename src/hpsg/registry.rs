//! Registry for HPSG types and features
//!
//! This module provides registries for types and features used in HPSG,
//! ensuring consistency across the grammar.

use std::collections::{HashMap, HashSet};
use crate::common::AtomicTypeRegistry;
use crate::hpsg::feature_structure::{FeatureStructure, TypedValue, FeatureType};
use crate::hpsg::type_hierarchy::TypeHierarchy;

/// Registry for HPSG types and features
#[derive(Debug, Clone)]
pub struct HPSGRegistry {
    /// Registry for atomic types
    pub type_registry: AtomicTypeRegistry,
    /// Registry for features
    pub feature_registry: HashSet<String>,
    /// Type hierarchy
    pub type_hierarchy: TypeHierarchy,
    /// Feature constraints (which features are appropriate for which types)
    pub feature_constraints: HashMap<String, HashSet<String>>,
    /// Counter for generating unique IDs
    next_id: usize,
}

impl HPSGRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            type_registry: AtomicTypeRegistry::new(),
            feature_registry: HashSet::new(),
            type_hierarchy: TypeHierarchy::new(),
            feature_constraints: HashMap::new(),
            next_id: 0,
        }
    }
    
    /// Register a new type
    pub fn register_type(&mut self, type_name: &str) {
        self.type_registry.register(type_name);
    }
    
    /// Register a new type with supertypes
    pub fn register_type_with_supertypes(&mut self, type_name: &str, supertypes: &[&str]) {
        self.type_registry.register(type_name);
        self.type_hierarchy.add_type_with_supertypes(type_name, supertypes);
    }
    
    /// Register a new feature
    pub fn register_feature(&mut self, feature_name: &str) {
        self.feature_registry.insert(feature_name.to_string());
    }
    
    /// Register a feature as appropriate for a type
    pub fn register_feature_for_type(&mut self, feature_name: &str, type_name: &str) {
        self.register_feature(feature_name);
        self.register_type(type_name);
        
        self.feature_constraints
            .entry(type_name.to_string())
            .or_insert_with(HashSet::new)
            .insert(feature_name.to_string());
    }
    
    /// Check if a type is registered
    pub fn is_type_registered(&self, type_name: &str) -> bool {
        self.type_registry.is_registered(type_name)
    }
    
    /// Check if a feature is registered
    pub fn is_feature_registered(&self, feature_name: &str) -> bool {
        self.feature_registry.contains(feature_name)
    }
    
    /// Check if a feature is appropriate for a type
    pub fn is_feature_appropriate_for_type(&self, feature_name: &str, type_name: &str) -> bool {
        if let Some(features) = self.feature_constraints.get(type_name) {
            features.contains(feature_name)
        } else {
            false
        }
    }
    
    /// Get the next available ID
    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    /// Create a standard HPSG registry with common types and features
    pub fn standard() -> Self {
        let mut registry = Self::new();
        
        // Register basic types
        registry.register_type("*top*");
        registry.register_type_with_supertypes("sign", &["*top*"]);
        registry.register_type_with_supertypes("word", &["sign"]);
        registry.register_type_with_supertypes("phrase", &["sign"]);
        
        // Register part-of-speech types
        registry.register_type_with_supertypes("pos", &["*top*"]);
        registry.register_type_with_supertypes("noun", &["pos"]);
        registry.register_type_with_supertypes("verb", &["pos"]);
        registry.register_type_with_supertypes("adj", &["pos"]);
        registry.register_type_with_supertypes("det", &["pos"]);
        registry.register_type_with_supertypes("prep", &["pos"]);
        
        // Register standard features
        registry.register_feature("PHON");
        registry.register_feature("SYNSEM");
        registry.register_feature("HEAD");
        registry.register_feature("SUBJ");
        registry.register_feature("COMPS");
        registry.register_feature("CONTENT");
        registry.register_feature("CONTEXT");
        
        // Register feature appropriateness
        registry.register_feature_for_type("PHON", "sign");
        registry.register_feature_for_type("SYNSEM", "sign");
        registry.register_feature_for_type("HEAD", "pos");
        registry.register_feature_for_type("SUBJ", "verb");
        registry.register_feature_for_type("COMPS", "verb");
        
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = HPSGRegistry::new();
        
        assert!(registry.type_registry.is_empty());
        assert!(registry.feature_registry.is_empty());
        assert!(registry.feature_constraints.is_empty());
    }
    
    #[test]
    fn test_register_type() {
        let mut registry = HPSGRegistry::new();
        
        registry.register_type("noun");
        
        assert!(registry.is_type_registered("noun"));
        assert!(!registry.is_type_registered("verb"));
    }
    
    #[test]
    fn test_register_feature() {
        let mut registry = HPSGRegistry::new();
        
        registry.register_feature("HEAD");
        
        assert!(registry.is_feature_registered("HEAD"));
        assert!(!registry.is_feature_registered("SUBJ"));
    }
    
    #[test]
    fn test_feature_appropriateness() {
        let mut registry = HPSGRegistry::new();
        
        registry.register_feature_for_type("HEAD", "noun");
        
        assert!(registry.is_feature_appropriate_for_type("HEAD", "noun"));
        assert!(!registry.is_feature_appropriate_for_type("SUBJ", "noun"));
    }
    
    #[test]
    fn test_standard_registry() {
        let registry = HPSGRegistry::standard();
        
        assert!(registry.is_type_registered("noun"));
        assert!(registry.is_type_registered("verb"));
        assert!(registry.is_feature_registered("HEAD"));
        assert!(registry.is_feature_registered("SUBJ"));
        assert!(registry.is_feature_appropriate_for_type("PHON", "sign"));
    }
    
    #[test]
    fn test_next_id() {
        let mut registry = HPSGRegistry::new();
        
        assert_eq!(registry.next_id(), 0);
        assert_eq!(registry.next_id(), 1);
        assert_eq!(registry.next_id(), 2);
    }
}