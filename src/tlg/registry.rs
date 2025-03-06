//! Registry for atomic types in Type-Logical Grammar

use std::collections::HashSet;

/// Registry for atomic types in Type-Logical Grammar
#[derive(Debug, Clone)]
pub struct AtomicTypeRegistry {
    /// Set of registered atomic type names
    types: HashSet<String>,
}

impl AtomicTypeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        AtomicTypeRegistry {
            types: HashSet::new(),
        }
    }
    
    /// Register a new atomic type
    pub fn register(&mut self, type_name: &str) {
        self.types.insert(type_name.to_string());
    }
    
    /// Check if a type is registered
    pub fn is_registered(&self, type_name: &str) -> bool {
        self.types.contains(type_name)
    }
    
    /// Get all registered types
    pub fn get_all_types(&self) -> Vec<String> {
        self.types.iter().cloned().collect()
    }
    
    /// Register multiple types at once
    pub fn register_multiple(&mut self, type_names: &[&str]) {
        for name in type_names {
            self.register(name);
        }
    }
    
    /// Remove a type from the registry
    pub fn remove(&mut self, type_name: &str) {
        self.types.remove(type_name);
    }
    
    /// Clear the registry
    pub fn clear(&mut self) {
        self.types.clear();
    }
    
    /// Get the number of registered types
    pub fn len(&self) -> usize {
        self.types.len()
    }
    
    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

impl Default for AtomicTypeRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        // Add standard linguistic types by default
        registry.register_multiple(&["s", "np", "n"]);
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = AtomicTypeRegistry::new();
        assert!(registry.is_empty());
        
        let default_registry = AtomicTypeRegistry::default();
        assert!(!default_registry.is_empty());
        assert!(default_registry.is_registered("s"));
        assert!(default_registry.is_registered("np"));
        assert!(default_registry.is_registered("n"));
    }
    
    #[test]
    fn test_register_atomic_type() {
        let mut registry = AtomicTypeRegistry::new();
        registry.register("NP");
        registry.register("S");
        
        assert!(registry.is_registered("NP"));
        assert!(registry.is_registered("S"));
        assert!(!registry.is_registered("PP"));
    }
    
    #[test]
    fn test_get_all_types() {
        let mut registry = AtomicTypeRegistry::new();
        registry.register("NP");
        registry.register("S");
        registry.register("N");
        
        let types = registry.get_all_types();
        assert_eq!(types.len(), 3);
        assert!(types.contains(&"NP".to_string()));
        assert!(types.contains(&"S".to_string()));
        assert!(types.contains(&"N".to_string()));
    }
    
    #[test]
    fn test_registry_operations() {
        let mut registry = AtomicTypeRegistry::new();
        registry.register_multiple(&["a", "b", "c"]);
        
        assert_eq!(registry.len(), 3);
        
        registry.remove("b");
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_registered("b"));
        
        registry.clear();
        assert!(registry.is_empty());
    }
}