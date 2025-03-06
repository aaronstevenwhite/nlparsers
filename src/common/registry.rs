//! Type registries for grammar formalisms

use std::collections::HashSet;

/// Registry for atomic types in grammar formalisms
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
        Self::new()
    }
}

/// Generic registry for any type of linguistic element
#[derive(Debug, Clone)]
pub struct Registry<T> {
    /// Map from names to elements
    elements: HashSet<T>,
}

impl<T: Clone + PartialEq + Eq + std::hash::Hash> Registry<T> {
    /// Create a new empty registry
    pub fn new() -> Self {
        Registry {
            elements: HashSet::new(),
        }
    }
    
    /// Register a new element
    pub fn register(&mut self, element: T) {
        self.elements.insert(element);
    }
    
    /// Check if an element is registered
    pub fn contains(&self, element: &T) -> bool {
        self.elements.contains(element)
    }
    
    /// Get all registered elements
    pub fn get_all(&self) -> Vec<T> {
        self.elements.iter().cloned().collect()
    }
    
    /// Remove an element from the registry
    pub fn remove(&mut self, element: &T) {
        self.elements.remove(element);
    }
    
    /// Clear the registry
    pub fn clear(&mut self) {
        self.elements.clear();
    }
    
    /// Get the number of registered elements
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    
    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<T: Clone + PartialEq + Eq + std::hash::Hash> Default for Registry<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_type_registry() {
        let mut registry = AtomicTypeRegistry::new();
        registry.register("NP");
        registry.register("S");
        
        assert!(registry.is_registered("NP"));
        assert!(registry.is_registered("S"));
        assert!(!registry.is_registered("PP"));
        
        assert_eq!(registry.len(), 2);
        
        registry.remove("NP");
        assert!(!registry.is_registered("NP"));
        assert_eq!(registry.len(), 1);
        
        registry.clear();
        assert!(registry.is_empty());
    }
    
    #[test]
    fn test_generic_registry() {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        struct Feature {
            name: String,
            value: String,
        }
        
        let mut registry = Registry::new();
        
        let feat1 = Feature { name: "num".to_string(), value: "sg".to_string() };
        let feat2 = Feature { name: "per".to_string(), value: "3".to_string() };
        
        registry.register(feat1.clone());
        registry.register(feat2.clone());
        
        assert!(registry.contains(&feat1));
        assert!(registry.contains(&feat2));
        assert_eq!(registry.len(), 2);
        
        registry.remove(&feat1);
        assert!(!registry.contains(&feat1));
        assert_eq!(registry.len(), 1);
        
        registry.clear();
        assert!(registry.is_empty());
    }
}