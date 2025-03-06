//! Feature structures and operations for linguistic features

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash;

/// Morphosyntactic feature value that can be used across different grammar formalisms
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureValue {
    /// Unspecified/underspecified value
    Unspecified,
    /// Atomic value (e.g., sg, pl, 3)
    Atomic(String),
    /// Set of possible values (e.g., {sg, pl})
    Set(Vec<String>),
    /// Complex value for nested features
    Complex(Box<FeatureStructure>),
    /// Variable for unification systems
    Variable(String),
}

impl fmt::Display for FeatureValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureValue::Unspecified => write!(f, "_"),
            FeatureValue::Atomic(s) => write!(f, "{}", s),
            FeatureValue::Set(set) => {
                write!(f, "{{")?;
                for (i, val) in set.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "}}")
            },
            FeatureValue::Complex(fs) => write!(f, "[{}]", fs),
            FeatureValue::Variable(v) => write!(f, "?{}", v),
        }
    }
}

/// Morphosyntactic feature structure used across grammar formalisms
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FeatureStructure {
    /// Map of feature names to values
    pub features: HashMap<String, FeatureValue>,
}

impl FeatureStructure {
    /// Create a new empty feature structure
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }
    
    /// Create a feature structure with a single feature
    pub fn with_feature(name: &str, value: FeatureValue) -> Self {
        let mut fs = Self::new();
        fs.add(name, value);
        fs
    }
    
    /// Add a feature to the structure
    pub fn add(&mut self, name: &str, value: FeatureValue) {
        self.features.insert(name.to_string(), value);
    }
    
    /// Get a feature value by name
    pub fn get(&self, name: &str) -> Option<&FeatureValue> {
        self.features.get(name)
    }
    
    /// Check if this feature structure unifies with another
    pub fn unifies_with(&self, other: &FeatureStructure) -> bool {
        // For each feature in this structure
        for (name, value) in &self.features {
            // If the other structure has this feature
            if let Some(other_value) = other.features.get(name) {
                // Check if the values unify
                if !Self::values_unify(value, other_value) {
                    return false;
                }
            }
        }

        // For each feature in the other structure not in this structure,
        // we implicitly assume it unifies (open-world assumption)
        true
    }

    /// Create a new feature structure by unifying two feature structures
    pub fn unify(&self, other: &FeatureStructure) -> Option<FeatureStructure> {
        if !self.unifies_with(other) {
            return None;
        }

        let mut result = self.clone();

        // Add features from other that are not in self
        for (name, value) in &other.features {
            if let Some(self_value) = self.features.get(name) {
                // Both structures have this feature, need to unify values
                if let Some(unified_value) = Self::unify_values(self_value, value) {
                    result.features.insert(name.clone(), unified_value);
                } else {
                    return None; // Unification failure
                }
            } else {
                // Only other has this feature, add it
                result.features.insert(name.clone(), value.clone());
            }
        }

        Some(result)
    }

    /// Check if two feature values unify
    pub fn values_unify(v1: &FeatureValue, v2: &FeatureValue) -> bool {
        match (v1, v2) {
            (FeatureValue::Unspecified, _) | (_, FeatureValue::Unspecified) => true,
            (FeatureValue::Atomic(s1), FeatureValue::Atomic(s2)) => s1 == s2,
            (FeatureValue::Set(set1), FeatureValue::Set(set2)) => {
                // Sets unify if they have a non-empty intersection
                set1.iter().any(|item| set2.contains(item))
            }
            (FeatureValue::Atomic(s), FeatureValue::Set(set)) |
            (FeatureValue::Set(set), FeatureValue::Atomic(s)) => {
                // Atomic value unifies with a set if it's a member
                set.contains(s)
            },
            (FeatureValue::Complex(fs1), FeatureValue::Complex(fs2)) => {
                // Complex feature structures unify if their internal features unify
                fs1.unifies_with(fs2)
            },
            (FeatureValue::Variable(_), _) | (_, FeatureValue::Variable(_)) => {
                // Variables can unify with anything (simplified for now)
                true
            },
            _ => false,
        }
    }

    /// Unify two feature values
    pub fn unify_values(v1: &FeatureValue, v2: &FeatureValue) -> Option<FeatureValue> {
        match (v1, v2) {
            (FeatureValue::Unspecified, _) => Some(v2.clone()),
            (_, FeatureValue::Unspecified) => Some(v1.clone()),
            (FeatureValue::Atomic(s1), FeatureValue::Atomic(s2)) => {
                if s1 == s2 {
                    Some(v1.clone())
                } else {
                    None // Cannot unify different atomic values
                }
            }
            (FeatureValue::Set(set1), FeatureValue::Set(set2)) => {
                // Intersection of the sets
                let intersection: Vec<String> = set1.iter()
                    .filter(|item| set2.contains(item))
                    .cloned()
                    .collect();
                if intersection.is_empty() {
                    None // Empty intersection means unification failure
                } else {
                    Some(FeatureValue::Set(intersection))
                }
            }
            (FeatureValue::Atomic(s), FeatureValue::Set(set)) => {
                if set.contains(s) {
                    Some(FeatureValue::Atomic(s.clone()))
                } else {
                    None
                }
            }
            (FeatureValue::Set(set), FeatureValue::Atomic(s)) => {
                if set.contains(s) {
                    Some(FeatureValue::Atomic(s.clone()))
                } else {
                    None
                }
            },
            (FeatureValue::Complex(fs1), FeatureValue::Complex(fs2)) => {
                fs1.unify(fs2).map(|fs| FeatureValue::Complex(Box::new(fs)))
            },
            (FeatureValue::Variable(_v), _) => {
                // Bind the variable to the value (simplified)
                Some(v2.clone())
            },
            (_, FeatureValue::Variable(_v)) => {
                // Bind the variable to the value (simplified)
                Some(v1.clone())
            },
            _ => None,
        }
    }
}

impl fmt::Display for FeatureStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.features.is_empty() {
            return Ok(());
        }

        write!(f, "[")?;
        let mut first = true;
        for (name, value) in &self.features {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}={}", name, value)?;
            first = false;
        }
        write!(f, "]")
    }
}

// Add Hash implementation for FeatureValue
impl hash::Hash for FeatureValue {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            FeatureValue::Unspecified => {
                // Hash a discriminant value for Unspecified
                0u8.hash(state);
            },
            FeatureValue::Atomic(s) => {
                // Hash a discriminant value for Atomic followed by the string
                1u8.hash(state);
                s.hash(state);
            },
            FeatureValue::Set(set) => {
                // Hash a discriminant value for Set
                2u8.hash(state);
                // Sort the set to ensure consistent hashing regardless of order
                let mut sorted_set = set.clone();
                sorted_set.sort();
                for item in sorted_set {
                    item.hash(state);
                }
            },
            FeatureValue::Complex(fs) => {
                // Hash a discriminant value for Complex
                3u8.hash(state);
                fs.hash(state);
            },
            FeatureValue::Variable(v) => {
                // Hash a discriminant value for Variable
                4u8.hash(state);
                v.hash(state);
            },
        }
    }
}

// Add Hash implementation for FeatureStructure
impl hash::Hash for FeatureStructure {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        // Create a sorted vector of entries to ensure consistent hashing
        let mut entries: Vec<_> = self.features.iter().collect();
        entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        
        // Hash each entry in order
        for (key, value) in entries {
            key.hash(state);
            value.hash(state);
        }
    }
}

/// Registry for features and their possible values
#[derive(Debug, Clone)]
pub struct FeatureRegistry {
    /// Feature names and their possible values
    pub features: HashMap<String, HashSet<String>>,
}

impl FeatureRegistry {
    /// Create a new empty feature registry
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }
    
    /// Register a new feature dimension and its possible values
    pub fn register_feature(&mut self, name: &str, values: &[&str]) {
        let value_set: HashSet<String> = values.iter().map(|v| v.to_string()).collect();
        self.features.insert(name.to_string(), value_set);
    }
    
    /// Check if a feature is registered
    pub fn is_feature_registered(&self, name: &str) -> bool {
        self.features.contains_key(name)
    }
    
    /// Check if a value is valid for a feature
    pub fn is_value_valid(&self, name: &str, value: &str) -> bool {
        if let Some(values) = self.features.get(name) {
            values.contains(value)
        } else {
            false
        }
    }
    
    /// Get all possible values for a feature
    pub fn get_values(&self, name: &str) -> Option<Vec<String>> {
        self.features.get(name).map(|set| set.iter().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_unification_basic() {
        // Test basic feature unification
        let mut feat1 = FeatureStructure::new();
        feat1.add("num", FeatureValue::Atomic("sg".to_string()));
        
        let mut feat2 = FeatureStructure::new();
        feat2.add("per", FeatureValue::Atomic("3".to_string()));
        
        // Features with no overlap should unify
        let unified = feat1.unify(&feat2);
        assert!(unified.is_some());
        
        let unified_feat = unified.unwrap();
        assert_eq!(unified_feat.get("num"), Some(&FeatureValue::Atomic("sg".to_string())));
        assert_eq!(unified_feat.get("per"), Some(&FeatureValue::Atomic("3".to_string())));
    }
    
    #[test]
    fn test_feature_unification_conflict() {
        // Test feature unification with conflicts
        let mut feat1 = FeatureStructure::new();
        feat1.add("num", FeatureValue::Atomic("sg".to_string()));
        
        let mut feat2 = FeatureStructure::new();
        feat2.add("num", FeatureValue::Atomic("pl".to_string()));
        
        // Conflicting values should not unify
        let unified = feat1.unify(&feat2);
        assert!(unified.is_none());
    }
    
    #[test]
    fn test_feature_unification_sets() {
        // Test feature unification with sets
        let mut feat1 = FeatureStructure::new();
        let set1: Vec<String> = ["sg".to_string(), "pl".to_string()].iter().cloned().collect();
        feat1.add("num", FeatureValue::Set(set1));
        
        let mut feat2 = FeatureStructure::new();
        feat2.add("num", FeatureValue::Atomic("sg".to_string()));
        // Set with compatible value should unify to the more specific one
        let unified = feat1.unify(&feat2);
        assert!(unified.is_some());
        
        let unified_feat = unified.unwrap();
        assert_eq!(unified_feat.get("num"), Some(&FeatureValue::Atomic("sg".to_string())));
    }
    
    #[test]
    fn test_feature_unification_complex() {
        // Test feature unification with complex values
        let mut inner1 = FeatureStructure::new();
        inner1.add("value", FeatureValue::Atomic("val1".to_string()));
        
        let mut feat1 = FeatureStructure::new();
        feat1.add("complex", FeatureValue::Complex(Box::new(inner1)));
        
        let mut inner2 = FeatureStructure::new();
        inner2.add("value", FeatureValue::Atomic("val1".to_string()));
        inner2.add("extra", FeatureValue::Atomic("extra_val".to_string()));
        
        let mut feat2 = FeatureStructure::new();
        feat2.add("complex", FeatureValue::Complex(Box::new(inner2)));
        
        // Complex values should unify if their features unify
        let unified = feat1.unify(&feat2);
        assert!(unified.is_some());
        
        if let Some(unified_feat) = unified {
            if let Some(FeatureValue::Complex(box_fs)) = unified_feat.get("complex") {
                assert_eq!(box_fs.get("value"), Some(&FeatureValue::Atomic("val1".to_string())));
                assert_eq!(box_fs.get("extra"), Some(&FeatureValue::Atomic("extra_val".to_string())));
            } else {
                panic!("Expected Complex feature type");
            }
        }
    }
}