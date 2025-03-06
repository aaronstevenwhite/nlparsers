//! Registry for atomic categories in LRFG
//!
//! This module provides a registry for atomic categories used in LRFG.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// An atomic category in LRFG
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AtomicCategory {
    /// Name of the category
    pub name: String,
    /// ID of the category
    pub id: usize,
}

impl AtomicCategory {
    /// Create a new atomic category
    pub fn new(name: &str, id: usize) -> Self {
        Self {
            name: name.to_string(),
            id,
        }
    }
}

/// Registry for atomic categories
#[derive(Debug, Clone)]
pub struct AtomicCategoryRegistry {
    /// Map from category names to IDs
    name_to_id: HashMap<String, usize>,
    /// Map from IDs to category names
    id_to_name: HashMap<usize, String>,
    /// Next available ID
    next_id: usize,
}

impl AtomicCategoryRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: HashMap::new(),
            next_id: 0,
        }
    }
    
    /// Register a category and get its ID
    pub fn register(&mut self, name: &str) -> usize {
        if let Some(id) = self.name_to_id.get(name) {
            *id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.name_to_id.insert(name.to_string(), id);
            self.id_to_name.insert(id, name.to_string());
            id
        }
    }
    
    /// Get the ID of a category
    pub fn get_id(&self, name: &str) -> Option<usize> {
        self.name_to_id.get(name).copied()
    }
    
    /// Get the name of a category
    pub fn get_name(&self, id: usize) -> Option<&str> {
        self.id_to_name.get(&id).map(|s| s.as_str())
    }
    
    /// Get a category by name
    pub fn get_category(&self, name: &str) -> Option<AtomicCategory> {
        self.get_id(name).map(|id| AtomicCategory::new(name, id))
    }
}

/// Global registry for atomic categories
pub static GLOBAL_REGISTRY: std::sync::OnceLock<Arc<RwLock<AtomicCategoryRegistry>>> = 
    std::sync::OnceLock::new();

/// Register a category in the global registry
pub fn register_global(name: &str) -> usize {
    let registry = GLOBAL_REGISTRY.get_or_init(|| {
        Arc::new(RwLock::new(AtomicCategoryRegistry::new()))
    });
    let mut registry_guard = registry.write().unwrap();
    registry_guard.register(name)
}

/// Get a category from the global registry
pub fn get_global_category(name: &str) -> Option<AtomicCategory> {
    let registry = GLOBAL_REGISTRY.get_or_init(|| {
        Arc::new(RwLock::new(AtomicCategoryRegistry::new()))
    });
    let registry_guard = registry.read().unwrap();
    registry_guard.get_category(name)
} 