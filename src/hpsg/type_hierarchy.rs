//! Type hierarchy system for Head-Driven Phrase Structure Grammar
//!
//! This module provides a type hierarchy implementation for HPSG,
//! which is an inheritance-based system for organizing linguistic types.

use std::fmt;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;

/// A type in the type hierarchy
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    /// Name of the type
    pub name: String,
    /// Parent types
    pub parents: Vec<String>,
    /// Child types
    pub children: Vec<String>,
    /// Features appropriate for this type
    pub features: HashSet<String>,
    /// Cache for subtype checking
    pub subtype_cache: RefCell<HashMap<String, bool>>,
    /// Cache for all subtypes
    pub all_subtypes_cache: RefCell<Option<HashSet<String>>>,
    /// Cache for appropriate features
    pub appropriate_features_cache: RefCell<Option<HashSet<String>>>,
}

impl Type {
    /// Create a new type with a name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parents: Vec::new(),
            children: Vec::new(),
            features: HashSet::new(),
            subtype_cache: RefCell::new(HashMap::new()),
            all_subtypes_cache: RefCell::new(None),
            appropriate_features_cache: RefCell::new(None),
        }
    }
    
    /// Create a type with supertypes
    pub fn with_supertypes(name: &str, supertypes: &[&str]) -> Self {
        let mut type_def = Self::new(name);
        for &supertype in supertypes {
            type_def.parents.push(supertype.to_string());
        }
        type_def
    }
    
    /// Add a supertype to this type
    pub fn add_supertype(&mut self, supertype: &str) {
        self.parents.push(supertype.to_string());
    }
    
    /// Add a subtype to this type
    pub fn add_subtype(&mut self, subtype: &str) {
        self.children.push(subtype.to_string());
    }
    
    /// Add an appropriate feature for this type
    pub fn add_appropriate_feature(&mut self, feature: &str) {
        self.features.insert(feature.to_string());
    }
    
    /// Add multiple appropriate features
    pub fn add_appropriate_features(&mut self, features: &[&str]) {
        for &feature in features {
            self.add_appropriate_feature(feature);
        }
    }
    
    /// Check if this type has a feature as appropriate
    pub fn has_appropriate_feature(&self, feature: &str) -> bool {
        self.features.contains(feature)
    }
    
    /// Check if this type is a subtype of another type
    pub fn is_subtype_of(&self, other: &str, hierarchy: &mut TypeHierarchy) -> bool {
        // Check cache first
        if let Some(result) = self.subtype_cache.borrow().get(other) {
            return *result;
        }
        
        // Special case: any type is a subtype of itself
        if self.name == other {
            return true;
        }
        
        // Check the cache
        if let Some(supertypes) = hierarchy.supertypes_cache.get(other) {
            return supertypes.contains(self.name.as_str());
        }
        
        // Compute the transitive closure of supertypes
        let mut all_supertypes = HashSet::new();
        hierarchy.collect_supertypes(other, &mut all_supertypes);
        
        // Cache the result
        let result = all_supertypes.contains(self.name.as_str());
        let mut cache_entry = all_supertypes.clone();
        cache_entry.insert(other.to_string()); // Include the type itself
        hierarchy.supertypes_cache.insert(other.to_string(), cache_entry);
        
        self.subtype_cache.borrow_mut().insert(other.to_string(), result);
        result
    }
    
    /// Get all subtypes of this type
    pub fn get_all_subtypes(&self, hierarchy: &TypeHierarchy) -> HashSet<String> {
        // Check cache first
        if let Some(subtypes) = self.all_subtypes_cache.borrow().as_ref() {
            return subtypes.clone();
        }
        
        // Compute the transitive closure of subtypes
        let mut all_subtypes = HashSet::new();
        hierarchy.collect_subtypes(self.name.as_str(), &mut all_subtypes);
        
        // Cache the result
        let result = all_subtypes.clone();
        let mut cache_entry = all_subtypes;
        cache_entry.insert(self.name.clone()); // Include the type itself
        self.all_subtypes_cache.borrow_mut().replace(cache_entry);
        
        result
    }
    
    /// Get all appropriate features for this type
    pub fn get_appropriate_features(&self, hierarchy: &TypeHierarchy) -> HashSet<String> {
        // Check cache first
        if let Some(features) = self.appropriate_features_cache.borrow().as_ref() {
            return features.clone();
        }
        
        let mut features = HashSet::new();
        
        // First get features from this type
        features.extend(self.features.clone());
        
        // Then get features from supertypes
        if let Some(supertypes) = hierarchy.supertypes_cache.get(&self.name) {
            for supertype in supertypes {
                if let Some(type_def) = hierarchy.types.get(supertype) {
                    features.extend(type_def.features.clone());
                }
            }
        } else {
            // Compute supertypes first if not cached
            let mut supertypes = HashSet::new();
            hierarchy.collect_supertypes(&self.name, &mut supertypes);
            
            for supertype in &supertypes {
                if let Some(type_def) = hierarchy.types.get(supertype) {
                    features.extend(type_def.features.clone());
                }
            }
        }
        
        self.appropriate_features_cache.borrow_mut().replace(features.clone());
        features
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        
        if !self.parents.is_empty() {
            write!(f, " < ")?;
            let supertypes: Vec<&String> = self.parents.iter().collect();
            for (i, supertype) in supertypes.iter().enumerate() {
                if i > 0 {
                    write!(f, " & ")?;
                }
                write!(f, "{}", supertype)?;
            }
        }
        
        if !self.features.is_empty() {
            write!(f, " [")?;
            let features: Vec<&String> = self.features.iter().collect();
            for (i, feature) in features.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", feature)?;
            }
            write!(f, "]")?;
        }
        
        Ok(())
    }
}

/// The type hierarchy for HPSG
#[derive(Debug, Clone)]
pub struct TypeHierarchy {
    /// Map of type names to type definitions
    pub types: HashMap<String, Type>,
    /// Cache of transitive subtypes for each type
    subtypes_cache: HashMap<String, HashSet<String>>,
    /// Cache of transitive supertypes for each type
    supertypes_cache: HashMap<String, HashSet<String>>,
}

impl TypeHierarchy {
    /// Create a new empty type hierarchy
    pub fn new() -> Self {
        // Create a hierarchy with the top type
        let mut hierarchy = Self {
            types: HashMap::new(),
            subtypes_cache: HashMap::new(),
            supertypes_cache: HashMap::new(),
        };
        
        // Add the *top* type (root of hierarchy)
        hierarchy.add_type(Type::new("*top*"));
        
        hierarchy
    }
    
    /// Add a type to the hierarchy
    pub fn add_type(&mut self, type_def: Type) {
        let type_name = type_def.name.clone();
        
        // Update subtypes of supertypes
        for supertype in &type_def.parents {
            if let Some(parent) = self.types.get_mut(supertype) {
                parent.add_subtype(&type_name);
            }
        }
        
        // Add the type
        self.types.insert(type_name, type_def);
        
        // Clear the caches since the hierarchy has changed
        self.subtypes_cache.clear();
        self.supertypes_cache.clear();
    }
    
    /// Add a type with supertypes to the hierarchy
    pub fn add_type_with_supertypes(&mut self, name: &str, supertypes: &[&str]) {
        let type_def = Type::with_supertypes(name, supertypes);
        self.add_type(type_def);
    }
    
    /// Get a type by name
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }
    
    /// Get a mutable reference to a type
    pub fn get_type_mut(&mut self, name: &str) -> Option<&mut Type> {
        self.types.get_mut(name)
    }
    
    /// Check if a type exists in the hierarchy
    pub fn has_type(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }
    
    /// Collect all supertypes (transitive closure) of a type
    fn collect_supertypes(&self, type_name: &str, result: &mut HashSet<String>) {
        if let Some(type_def) = self.types.get(type_name) {
            for supertype in &type_def.parents {
                result.insert(supertype.clone());
                self.collect_supertypes(supertype, result);
            }
        }
    }
    
    /// Collect all subtypes (transitive closure) of a type
    fn collect_subtypes(&self, type_name: &str, result: &mut HashSet<String>) {
        if let Some(type_def) = self.types.get(type_name) {
            for subtype in &type_def.children {
                result.insert(subtype.clone());
                self.collect_subtypes(subtype, result);
            }
        }
    }
    
    /// Create a basic linguistic type hierarchy
    pub fn linguistic() -> Self {
        let mut hierarchy = Self::new();
        
        // Basic linguistic types
        hierarchy.add_type_with_supertypes("sign", &["*top*"]);
        hierarchy.add_type_with_supertypes("synsem", &["*top*"]);
        hierarchy.add_type_with_supertypes("cat", &["*top*"]);
        hierarchy.add_type_with_supertypes("head", &["*top*"]);
        
        // Sign subtypes
        hierarchy.add_type_with_supertypes("word", &["sign"]);
        hierarchy.add_type_with_supertypes("phrase", &["sign"]);
        
        // Head subtypes
        hierarchy.add_type_with_supertypes("noun", &["head"]);
        hierarchy.add_type_with_supertypes("verb", &["head"]);
        hierarchy.add_type_with_supertypes("adj", &["head"]);
        hierarchy.add_type_with_supertypes("prep", &["head"]);
        hierarchy.add_type_with_supertypes("det", &["head"]);
        
        // Valence types
        hierarchy.add_type_with_supertypes("val", &["*top*"]);
        hierarchy.add_type_with_supertypes("subj", &["val"]);
        hierarchy.add_type_with_supertypes("comps", &["val"]);
        
        // Add appropriate features
        let features = ["PHON", "SYNSEM", "DAUGHTERS"];
        if let Some(sign) = hierarchy.get_type_mut("sign") {
            sign.add_appropriate_features(&features);
        }
        
        let features = ["LOCAL", "NON-LOCAL"];
        if let Some(synsem) = hierarchy.get_type_mut("synsem") {
            synsem.add_appropriate_features(&features);
        }
        
        let features = ["HEAD", "SUBJ", "COMPS"];
        if let Some(cat) = hierarchy.get_type_mut("cat") {
            cat.add_appropriate_features(&features);
        }
        
        hierarchy
    }
    
    /// Check if two types are compatible (one is a subtype of the other or they are the same)
    pub fn is_compatible(&self, type1: &str, type2: &str) -> bool {
        // Types are compatible if they're the same
        if type1 == type2 {
            return true;
        }
        
        // Check if type1 is a subtype of type2
        if let Some(supertypes) = self.supertypes_cache.get(type1) {
            if supertypes.contains(type2) {
                return true;
            }
        } else {
            let mut supertypes = HashSet::new();
            self.collect_supertypes(type1, &mut supertypes);
            if supertypes.contains(type2) {
                return true;
            }
        }
        
        // Check if type2 is a subtype of type1
        if let Some(supertypes) = self.supertypes_cache.get(type2) {
            if supertypes.contains(type1) {
                return true;
            }
        } else {
            let mut supertypes = HashSet::new();
            self.collect_supertypes(type2, &mut supertypes);
            if supertypes.contains(type1) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if type1 is more specific than type2 (type1 is a subtype of type2)
    pub fn is_more_specific(&self, type1: &str, type2: &str) -> bool {
        // A type is not more specific than itself
        if type1 == type2 {
            return false;
        }
        
        // Check if type1 is a subtype of type2
        if let Some(supertypes) = self.supertypes_cache.get(type1) {
            if supertypes.contains(type2) {
                return true;
            }
        } else {
            let mut supertypes = HashSet::new();
            self.collect_supertypes(type1, &mut supertypes);
            if supertypes.contains(type2) {
                return true;
            }
        }
        
        false
    }
}