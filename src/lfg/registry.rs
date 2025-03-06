//! Registry for atomic categories in Lexical-Functional Grammar
//!
//! This module provides a registry for atomic categories in LFG.

use std::collections::HashSet;
use crate::lfg::c_structure::Category;

/// Registry for atomic categories in LFG
#[derive(Debug, Clone)]
pub struct AtomicCategoryRegistry {
    /// Set of registered category names
    categories: HashSet<String>,
}

impl AtomicCategoryRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            categories: HashSet::new(),
        }
    }
    
    /// Register a new category
    pub fn register(&mut self, name: &str) {
        self.categories.insert(name.to_string());
    }
    
    /// Register multiple categories at once
    pub fn register_multiple(&mut self, names: &[&str]) {
        for name in names {
            self.register(name);
        }
    }
    
    /// Check if a category is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.categories.contains(name)
    }
    
    /// Get all registered categories
    pub fn get_all(&self) -> Vec<String> {
        self.categories.iter().cloned().collect()
    }
    
    /// Remove a category from the registry
    pub fn remove(&mut self, name: &str) {
        self.categories.remove(name);
    }
    
    /// Clear the registry
    pub fn clear(&mut self) {
        self.categories.clear();
    }
    
    /// Get the number of registered categories
    pub fn len(&self) -> usize {
        self.categories.len()
    }
    
    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }
    
    /// Create a category if it's registered
    pub fn create_category(&self, name: &str) -> Option<Category> {
        if self.is_registered(name) {
            Some(Category::new(name))
        } else {
            None
        }
    }
}

impl Default for AtomicCategoryRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        // Register standard categories
        registry.register_multiple(&["S", "NP", "VP", "N", "V", "Det", "A", "Adv", "P", "PP"]);
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = AtomicCategoryRegistry::new();
        assert!(registry.is_empty());
        
        let default_registry = AtomicCategoryRegistry::default();
        assert!(!default_registry.is_empty());
        assert!(default_registry.is_registered("S"));
        assert!(default_registry.is_registered("NP"));
        assert!(default_registry.is_registered("VP"));
    }
    
    #[test]
    fn test_registration() {
        let mut registry = AtomicCategoryRegistry::new();
        registry.register("X");
        registry.register("Y");
        
        assert!(registry.is_registered("X"));
        assert!(registry.is_registered("Y"));
        assert!(!registry.is_registered("Z"));
        
        assert_eq!(registry.len(), 2);
    }
    
    #[test]
    fn test_register_multiple() {
        let mut registry = AtomicCategoryRegistry::new();
        registry.register_multiple(&["A", "B", "C"]);
        
        assert!(registry.is_registered("A"));
        assert!(registry.is_registered("B"));
        assert!(registry.is_registered("C"));
        
        assert_eq!(registry.len(), 3);
    }
    
    #[test]
    fn test_remove() {
        let mut registry = AtomicCategoryRegistry::new();
        registry.register_multiple(&["A", "B", "C"]);
        
        registry.remove("B");
        
        assert!(registry.is_registered("A"));
        assert!(!registry.is_registered("B"));
        assert!(registry.is_registered("C"));
        
        assert_eq!(registry.len(), 2);
    }
    
    #[test]
    fn test_clear() {
        let mut registry = AtomicCategoryRegistry::new();
        registry.register_multiple(&["A", "B", "C"]);
        
        registry.clear();
        
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
    
    #[test]
    fn test_create_category() {
        let registry = AtomicCategoryRegistry::default();
        
        // Create a registered category
        let category = registry.create_category("NP");
        assert!(category.is_some());
        
        let np = category.unwrap();
        assert_eq!(np.name, "NP");
        
        // Try to create an unregistered category
        let category = registry.create_category("XX");
        assert!(category.is_none());
    }
}