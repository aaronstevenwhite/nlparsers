//! Feature structures for Head-Driven Phrase Structure Grammar
//!
//! This module provides typed feature structures, which are central to HPSG.
//! They are recursive attribute-value matrices with type inheritance.

use std::fmt;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::cell::RefCell;
use crate::common::{FeatureStructure as CommonFeatureStructure, FeatureValue as CommonFeatureValue};
use crate::hpsg::type_hierarchy::TypeHierarchy;

/// A type for feature values in HPSG
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FeatureType {
    /// Atomic string value (e.g., "3sg", "nom")
    String(String),
    /// Boolean value
    Bool(bool),
    /// Integer value (e.g., for indices)
    Integer(i64),
    /// List of values
    List(Vec<TypedValue>),
    /// Set of values
    Set(Vec<TypedValue>),
    /// Reference to another feature structure (for structure sharing and reentrancy)
    Reference(usize),
    /// Empty (underspecified) value
    Empty,
}

impl fmt::Display for FeatureType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureType::String(s) => write!(f, "\"{}\"", s),
            FeatureType::Bool(b) => write!(f, "{}", b),
            FeatureType::Integer(i) => write!(f, "{}", i),
            FeatureType::List(items) => {
                write!(f, "< ")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, " >")
            },
            FeatureType::Set(items) => {
                write!(f, "{{ ")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, " }}")
            },
            FeatureType::Reference(index) => write!(f, "#{}", index),
            FeatureType::Empty => write!(f, "_"),
        }
    }
}

/// A typed value in a feature structure
#[derive(Debug, Clone)]
pub struct TypedValue {
    /// The type in the type hierarchy
    pub type_name: String,
    /// The actual value
    pub value: FeatureType,
    /// Unique identifier for this value (used for references)
    pub id: usize,
}

impl PartialEq for TypedValue {
    fn eq(&self, other: &Self) -> bool {
        self.type_name == other.type_name && self.value == other.value
    }
}

impl Eq for TypedValue {}

impl Hash for TypedValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_name.hash(state);
        self.value.hash(state);
    }
}

impl fmt::Display for TypedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.type_name, self.value)
    }
}

/// A typed feature structure in HPSG
#[derive(Debug, Clone)]
pub struct FeatureStructure {
    /// The type in the type hierarchy
    pub type_name: String,
    /// Feature-value pairs
    pub features: HashMap<String, TypedValue>,
    /// Unique identifier for this structure
    pub id: usize,
    /// Reference to the type hierarchy
    pub type_hierarchy: Option<Rc<RefCell<TypeHierarchy>>>,
}

impl PartialEq for FeatureStructure {
    fn eq(&self, other: &Self) -> bool {
        self.type_name == other.type_name && self.features == other.features
    }
}

impl Eq for FeatureStructure {}

impl Hash for FeatureStructure {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_name.hash(state);
        
        // Sort keys for deterministic hashing
        let mut keys: Vec<&String> = self.features.keys().collect();
        keys.sort();
        
        for key in keys {
            key.hash(state);
            self.features.get(key).unwrap().hash(state);
        }
    }
}

impl fmt::Display for FeatureStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} ", self.type_name)?;
        
        let mut first = true;
        for (key, value) in &self.features {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}={}", key, value)?;
            first = false;
        }
        
        write!(f, "]")
    }
}

impl FeatureStructure {
    /// Create a new empty feature structure with a type
    pub fn new(type_name: &str, id: usize) -> Self {
        Self {
            type_name: type_name.to_string(),
            features: HashMap::new(),
            id,
            type_hierarchy: None,
        }
    }
    
    /// Create a feature structure with a type hierarchy
    pub fn with_type_hierarchy(type_name: &str, id: usize, hierarchy: Rc<RefCell<TypeHierarchy>>) -> Self {
        Self {
            type_name: type_name.to_string(),
            features: HashMap::new(),
            id,
            type_hierarchy: Some(hierarchy),
        }
    }
    
    /// Set a feature value
    pub fn set(&mut self, name: &str, value: TypedValue) {
        self.features.insert(name.to_string(), value);
    }
    
    /// Get a feature value
    pub fn get(&self, name: &str) -> Option<&TypedValue> {
        self.features.get(name)
    }
    
    /// Get a mutable reference to a feature value
    pub fn get_mut(&mut self, name: &str) -> Option<&mut TypedValue> {
        self.features.get_mut(name) // Remove `mut` here
    }
    
    /// Check if a feature exists
    pub fn has_feature(&self, name: &str) -> bool {
        self.features.contains_key(name)
    }
    
    /// Follow a feature path (e.g., "SYNSEM.LOCAL.CAT.HEAD")
    pub fn follow_path(&self, path: &str) -> Option<&TypedValue> {
        let components: Vec<&str> = path.split('.').collect();
        
        let current_fs = self;
        
        for (i, &component) in components.iter().enumerate() {
            if i == components.len() - 1 {
                // Last component is a feature name
                return current_fs.get(component);
            } else {
                // Intermediate component should be a feature structure
                match current_fs.get(component) {
                    Some(TypedValue { value: FeatureType::Reference(_ref_id), .. }) => {
                        // TODO: Follow reference (would need a reference map)
                        return None;
                    },
                    Some(_typed_value) => {
                        // We can't continue if this isn't a reference to another feature structure
                        return None;
                    },
                    None => return None,
                }
            }
        }
        
        None
    }
    
    /// Create from a common FeatureStructure
    pub fn from_common(common: &CommonFeatureStructure, id: usize) -> Self {
        let mut fs = Self::new("*top*", id);
        
        for (name, value) in &common.features {
            match value {
                CommonFeatureValue::Atomic(s) => {
                    fs.set(name, TypedValue {
                        type_name: "string".to_string(),
                        value: FeatureType::String(s.clone()),
                        id: fs.next_id(),
                    });
                },
                CommonFeatureValue::Set(set) => {
                    let mut hpsg_set = Vec::new();
                    for s in set {
                        hpsg_set.push(TypedValue {
                            type_name: "string".to_string(),
                            value: FeatureType::String(s.clone()),
                            id: fs.next_id(),
                        });
                    }
                    fs.set(name, TypedValue {
                        type_name: "set".to_string(),
                        value: FeatureType::Set(hpsg_set),
                        id: fs.next_id(),
                    });
                },
                CommonFeatureValue::Unspecified => {
                    fs.set(name, TypedValue {
                        type_name: "empty".to_string(),
                        value: FeatureType::Empty,
                        id: fs.next_id(),
                    });
                },
                CommonFeatureValue::Complex(box_fs) => {
                    // Convert the nested feature structure recursively
                    let nested_fs = Self::from_common(box_fs, fs.next_id());
                    // Store a reference to the nested structure
                    fs.set(name, TypedValue {
                        type_name: "complex".to_string(),
                        value: FeatureType::Reference(nested_fs.id),
                        id: fs.next_id(),
                    });
                    // In a real implementation, you would need to store the nested structure somewhere
                    // This is a simplification
                },
                CommonFeatureValue::Variable(var) => {
                    // For variables, we could use a string representation
                    fs.set(name, TypedValue {
                        type_name: "variable".to_string(),
                        value: FeatureType::String(format!("?{}", var)),
                        id: fs.next_id(),
                    });
                },
            }
        }
        
        fs
    }
    
    /// Convert to a common FeatureStructure for interoperability
    pub fn to_common(&self) -> CommonFeatureStructure {
        let mut common = CommonFeatureStructure::new();
        
        for (name, value) in &self.features {
            match &value.value {
                FeatureType::String(s) => {
                    common.add(name, CommonFeatureValue::Atomic(s.clone()));
                },
                FeatureType::Bool(b) => {
                    common.add(name, CommonFeatureValue::Atomic(b.to_string()));
                },
                FeatureType::Integer(i) => {
                    common.add(name, CommonFeatureValue::Atomic(i.to_string()));
                },
                FeatureType::List(items) => {
                    // Convert list to set for compatibility
                    let mut set = Vec::new();
                    for item in items {
                        if let FeatureType::String(s) = &item.value {
                            set.push(s.clone());
                        }
                        // Other types could be handled as needed
                    }
                    common.add(name, CommonFeatureValue::Set(set));
                },
                FeatureType::Set(items) => {
                    // Convert set items to strings
                    let mut set = Vec::new();
                    for item in items {
                        if let FeatureType::String(s) = &item.value {
                            set.push(s.clone());
                        }
                        // Other types could be handled as needed
                    }
                    common.add(name, CommonFeatureValue::Set(set));
                },
                FeatureType::Reference(_ref_id) => {
                    // For references, we would need a way to look up the referenced structure
                    // This is a simplification
                    common.add(name, CommonFeatureValue::Unspecified);
                },
                FeatureType::Empty => {
                    common.add(name, CommonFeatureValue::Unspecified);
                },
            }
        }
        
        common
    }
    
    /// Check if the feature structure has no features
    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }
    
    /// Generate a unique ID for a new value
    fn next_id(&self) -> usize {
        self.id + self.features.len() + 1
    }
    
    /// Get the next available ID for a new value (public version)
    pub fn get_next_id(&self) -> usize {
        self.next_id()
    }
    
    /// Unify this feature structure with another
    pub fn unify(&self, other: &FeatureStructure) -> Option<FeatureStructure> {
        // Check type compatibility
        if !self.is_type_compatible(&self.type_name, &other.type_name) {
            return None;
        }
        
        // Choose the most specific type
        let result_type = self.most_specific_type(&self.type_name, &other.type_name);
        
        let mut result = FeatureStructure::new(&result_type, self.id.max(other.id) + 1);
        if let Some(hierarchy) = &self.type_hierarchy {
            result.type_hierarchy = Some(Rc::clone(hierarchy));
        }
        
        // Unify feature values
        let mut all_features = HashSet::new();
        for key in self.features.keys() {
            all_features.insert(key.clone());
        }
        for key in other.features.keys() {
            all_features.insert(key.clone());
        }
        
        for feature in all_features {
            match (self.get(&feature), other.get(&feature)) {
                (Some(val1), Some(val2)) => {
                    // Both have the feature, unify the values
                    if let Some(unified) = self.unify_values(val1, val2) {
                        result.set(&feature, unified);
                    } else {
                        return None; // Unification failure
                    }
                },
                (Some(val), None) => {
                    // Only in self
                    result.set(&feature, val.clone());
                },
                (None, Some(val)) => {
                    // Only in other
                    result.set(&feature, val.clone());
                },
                (None, None) => {
                    // Shouldn't happen
                },
            }
        }
        
        Some(result)
    }
    
    /// Check if two types are compatible
    fn is_type_compatible(&self, type1: &str, type2: &str) -> bool {
        if let Some(hierarchy) = &self.type_hierarchy {
            let hierarchy_ref = hierarchy.borrow();
            return hierarchy_ref.is_compatible(type1, type2);
        }
        
        // Without a type hierarchy, only identical types are compatible
        type1 == type2
    }
    
    /// Determine the most specific type between two types
    fn most_specific_type(&self, type1: &str, type2: &str) -> String {
        if let Some(hierarchy) = &self.type_hierarchy {
            let hierarchy_ref = hierarchy.borrow();
            if hierarchy_ref.is_more_specific(type1, type2) {
                return type1.to_string();
            } else if hierarchy_ref.is_more_specific(type2, type1) {
                return type2.to_string();
            }
        }
        
        // If no hierarchy or types are unrelated, use the first type
        type1.to_string()
    }
    
    /// Unify two typed values
    fn unify_values(&self, val1: &TypedValue, val2: &TypedValue) -> Option<TypedValue> {
        // Check type compatibility
        if !self.is_type_compatible(&val1.type_name, &val2.type_name) {
            return None;
        }
        
        // Choose the most specific type
        let result_type = self.most_specific_type(&val1.type_name, &val2.type_name);
        
        // Unify the values
        match (&val1.value, &val2.value) {
            (FeatureType::String(s1), FeatureType::String(s2)) => {
                if s1 == s2 {
                    Some(TypedValue {
                        type_name: result_type,
                        value: FeatureType::String(s1.clone()),
                        id: val1.id.max(val2.id) + 1,
                    })
                } else {
                    None // String values don't match
                }
            },
            (FeatureType::Bool(b1), FeatureType::Bool(b2)) => {
                if b1 == b2 {
                    Some(TypedValue {
                        type_name: result_type,
                        value: FeatureType::Bool(*b1),
                        id: val1.id.max(val2.id) + 1,
                    })
                } else {
                    None // Boolean values don't match
                }
            },
            (FeatureType::Integer(i1), FeatureType::Integer(i2)) => {
                if i1 == i2 {
                    Some(TypedValue {
                        type_name: result_type,
                        value: FeatureType::Integer(*i1),
                        id: val1.id.max(val2.id) + 1,
                    })
                } else {
                    None // Integer values don't match
                }
            },
            (FeatureType::List(list1), FeatureType::List(list2)) => {
                // For lists, we could implement a more sophisticated unification
                // For now, just check if they're identical
                if list1 == list2 {
                    Some(TypedValue {
                        type_name: result_type,
                        value: FeatureType::List(list1.clone()),
                        id: val1.id.max(val2.id) + 1,
                    })
                } else {
                    None // Lists don't match
                }
            },
            (FeatureType::Set(set1), FeatureType::Set(set2)) => {
                // For sets, take the intersection (similar to common FeatureValue)
                let mut intersection = Vec::new();
                for item1 in set1 {
                    for item2 in set2 {
                        if let Some(unified) = self.unify_values(item1, item2) {
                            if !intersection.contains(&unified) {
                                intersection.push(unified);
                            }
                        }
                    }
                }
                
                if !intersection.is_empty() {
                    Some(TypedValue {
                        type_name: result_type,
                        value: FeatureType::Set(intersection),
                        id: val1.id.max(val2.id) + 1,
                    })
                } else {
                    None // Empty intersection means unification failure
                }
            },
            (FeatureType::Reference(ref1), FeatureType::Reference(ref2)) => {
                // For references, we would need to check if the referenced structures unify
                // This is a simplification
                if ref1 == ref2 {
                    Some(TypedValue {
                        type_name: result_type,
                        value: FeatureType::Reference(*ref1),
                        id: val1.id.max(val2.id) + 1,
                    })
                } else {
                    None // References don't match
                }
            },
            (FeatureType::Empty, value) | (value, FeatureType::Empty) => {
                // Empty unifies with anything, result is the non-empty value
                Some(TypedValue {
                    type_name: result_type,
                    value: value.clone(),
                    id: val1.id.max(val2.id) + 1,
                })
            },
            _ => None, // Incompatible value types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_structure_creation() {
        let fs = FeatureStructure::new("phrase", 1);
        
        assert_eq!(fs.type_name, "phrase");
        assert!(fs.features.is_empty());
        assert_eq!(fs.id, 1);
        assert!(fs.type_hierarchy.is_none());
    }
    
    #[test]
    fn test_setting_features() {
        let mut fs = FeatureStructure::new("phrase", 1);
        
        fs.set("HEAD", TypedValue {
            type_name: "noun".to_string(),
            value: FeatureType::String("noun".to_string()),
            id: 2,
        });
        
        assert!(fs.has_feature("HEAD"));
        assert!(!fs.has_feature("COMP"));
        
        let head = fs.get("HEAD").unwrap();
        assert_eq!(head.type_name, "noun");
        
        match &head.value {
            FeatureType::String(s) => assert_eq!(s, "noun"),
            _ => panic!("Expected string value"),
        }
    }
    
    #[test]
    fn test_feature_structure_display() {
        let mut fs = FeatureStructure::new("phrase", 1);
        
        fs.set("HEAD", TypedValue {
            type_name: "noun".to_string(),
            value: FeatureType::String("noun".to_string()),
            id: 2,
        });
        
        fs.set("NUM", TypedValue {
            type_name: "number".to_string(),
            value: FeatureType::String("sg".to_string()),
            id: 3,
        });
        
        let display = format!("{}", fs);
        assert!(display.contains("phrase"));
        assert!(display.contains("HEAD=noun:"));
        assert!(display.contains("NUM=number:"));
    }
    
    #[test]
    fn test_simple_unification() {
        let mut fs1 = FeatureStructure::new("phrase", 1);
        fs1.set("HEAD", TypedValue {
            type_name: "noun".to_string(),
            value: FeatureType::String("noun".to_string()),
            id: 2,
        });
        
        let mut fs2 = FeatureStructure::new("phrase", 3);
        fs2.set("NUM", TypedValue {
            type_name: "number".to_string(),
            value: FeatureType::String("sg".to_string()),
            id: 4,
        });
        
        let unified = fs1.unify(&fs2);
        assert!(unified.is_some());
        
        let result = unified.unwrap();
        assert_eq!(result.type_name, "phrase");
        assert!(result.has_feature("HEAD"));
        assert!(result.has_feature("NUM"));
    }
    
    #[test]
    fn test_unification_failure() {
        let mut fs1 = FeatureStructure::new("phrase", 1);
        fs1.set("NUM", TypedValue {
            type_name: "number".to_string(),
            value: FeatureType::String("sg".to_string()),
            id: 2,
        });
        
        let mut fs2 = FeatureStructure::new("phrase", 3);
        fs2.set("NUM", TypedValue {
            type_name: "number".to_string(),
            value: FeatureType::String("pl".to_string()),
            id: 4,
        });
        
        let unified = fs1.unify(&fs2);
        assert!(unified.is_none()); // Unification should fail
    }
    
    #[test]
    fn test_conversion_to_common() {
        let mut fs = FeatureStructure::new("phrase", 1);
        
        fs.set("HEAD", TypedValue {
            type_name: "noun".to_string(),
            value: FeatureType::String("noun".to_string()),
            id: 2,
        });
        
        fs.set("NUM", TypedValue {
            type_name: "number".to_string(),
            value: FeatureType::String("sg".to_string()),
            id: 3,
        });
        
        let common = fs.to_common();
        
        assert!(common.features.contains_key("HEAD"));
        assert!(common.features.contains_key("NUM"));
        
        match common.features.get("HEAD").unwrap() {
            CommonFeatureValue::Atomic(s) => assert_eq!(s, "noun"),
            _ => panic!("Expected atomic value"),
        }
    }
}