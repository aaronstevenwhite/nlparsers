//! CCG category implementation

use std::fmt;
use std::hash::{Hash, Hasher};
use crate::common::{FeatureStructure, FeatureValue};

/// The core syntactic category types in CCG, enhanced with morphosyntactic features
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CCGCategory {
    /// Atomic categories like S, NP, N
    Atomic(String, FeatureStructure),
    /// Forward slash category (X/Y)
    Forward(Box<CCGCategory>, Box<CCGCategory>),
    /// Backward slash category (X\Y)
    Backward(Box<CCGCategory>, Box<CCGCategory>),
}

// Manual implementation of Hash for CCGCategory
impl Hash for CCGCategory {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            CCGCategory::Atomic(s, _features) => {
                // Hash discriminant and string, but ignore features
                0.hash(state);  // Discriminant for Atomic
                s.hash(state);
                // We don't hash features since FeatureStructure doesn't implement Hash
                // This means two categories with same name but different features will hash the same
            }
            CCGCategory::Forward(x, y) => {
                // Hash discriminant and both subcategories
                1.hash(state);  // Discriminant for Forward
                x.hash(state);
                y.hash(state);
            }
            CCGCategory::Backward(x, y) => {
                // Hash discriminant and both subcategories
                2.hash(state);  // Discriminant for Backward
                x.hash(state);
                y.hash(state);
            }
        }
    }
}

impl fmt::Display for CCGCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CCGCategory::Atomic(s, features) => {
                write!(f, "{}", s)?;
                if !features.features.is_empty() {
                    write!(f, "{}", features)?;
                }
                Ok(())
            }
            CCGCategory::Forward(x, y) => {
                if matches!(**x, CCGCategory::Atomic(_, _)) {
                    write!(f, "{}/{}", x, y)
                } else {
                    write!(f, "({})/{}", x, y)
                }
            }
            CCGCategory::Backward(x, y) => {
                if matches!(**x, CCGCategory::Atomic(_, _)) {
                    write!(f, "{}\\{}", x, y)
                } else {
                    write!(f, "({})\\{}", x, y)
                }
            }
        }
    }
}

impl CCGCategory {
    /// Create a new atomic category from a string
    pub fn atomic(name: &str) -> Self {
        CCGCategory::Atomic(name.to_string(), FeatureStructure::new())
    }
    
    /// Create an atomic category with features
    pub fn atomic_with_features(name: &str, features: FeatureStructure) -> Self {
        CCGCategory::Atomic(name.to_string(), features)
    }
    
    /// Create a forward slash category (X/Y)
    pub fn forward(left: CCGCategory, right: CCGCategory) -> Self {
        CCGCategory::Forward(Box::new(left), Box::new(right))
    }

    /// Create a backward slash category (X\Y)
    pub fn backward(left: CCGCategory, right: CCGCategory) -> Self {
        CCGCategory::Backward(Box::new(left), Box::new(right))
    }
    
    /// Convenience method for creating S category
    pub fn s() -> Self {
        Self::atomic("S")
    }

    /// Convenience method for creating NP category
    pub fn np() -> Self {
        Self::atomic("NP")
    }

    /// Convenience method for creating N category
    pub fn n() -> Self {
        Self::atomic("N")
    }
    
    /// Create a noun with number feature
    pub fn n_with_number(number: &str) -> Self {
        let mut features = FeatureStructure::new();
        features.add("num", FeatureValue::Atomic(number.to_string()));
        Self::atomic_with_features("N", features)
    }
    
    /// Create a noun phrase with case and number features
    pub fn np_with_features(case: &str, number: &str) -> Self {
        let mut features = FeatureStructure::new();
        features.add("case", FeatureValue::Atomic(case.to_string()));
        features.add("num", FeatureValue::Atomic(number.to_string()));
        Self::atomic_with_features("NP", features)
    }
    
    /// Create a sentence category with agreement features
    pub fn s_with_agreement(subject_num: &str, subject_person: &str) -> Self {
        let mut features = FeatureStructure::new();
        features.add("s_num", FeatureValue::Atomic(subject_num.to_string()));
        features.add("s_per", FeatureValue::Atomic(subject_person.to_string()));
        Self::atomic_with_features("S", features)
    }
    
    /// Get the feature structure from an atomic category
    pub fn get_features(&self) -> Option<&FeatureStructure> {
        match self {
            CCGCategory::Atomic(_, features) => Some(features),
            _ => None,
        }
    }
    
    /// Unify this category with another
    pub fn unify(&self, other: &CCGCategory) -> Option<CCGCategory> {
        match (self, other) {
            (CCGCategory::Atomic(s1, f1), CCGCategory::Atomic(s2, f2)) => {
                if s1 != s2 {
                    return None;
                }
                
                // Unify feature structures
                if let Some(unified_features) = f1.unify(f2) {
                    Some(CCGCategory::Atomic(s1.clone(), unified_features))
                } else {
                    None
                }
            }
            (CCGCategory::Forward(x1, y1), CCGCategory::Forward(x2, y2)) => {
                // Recursively unify components
                if let (Some(unified_x), Some(unified_y)) = (x1.unify(x2), y1.unify(y2)) {
                    Some(CCGCategory::Forward(Box::new(unified_x), Box::new(unified_y)))
                } else {
                    None
                }
            }
            (CCGCategory::Backward(x1, y1), CCGCategory::Backward(x2, y2)) => {
                // Recursively unify components
                if let (Some(unified_x), Some(unified_y)) = (x1.unify(x2), y1.unify(y2)) {
                    Some(CCGCategory::Backward(Box::new(unified_x), Box::new(unified_y)))
                } else {
                    None
                }
            }
            _ => None, // Different category types don't unify
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{FeatureStructure, FeatureValue};

    #[test]
    fn test_category_display() {
        let np = CCGCategory::np();
        let n = CCGCategory::n();
        let det = CCGCategory::forward(np.clone(), n.clone());
        
        assert_eq!(np.to_string(), "NP");
        assert_eq!(det.to_string(), "NP/N");
        
        let s = CCGCategory::s();
        let iv = CCGCategory::backward(s.clone(), np.clone());
        let tv = CCGCategory::backward(iv.clone(), np.clone());
        
        assert_eq!(iv.to_string(), "S\\NP");
        assert_eq!(tv.to_string(), "(S\\NP)\\NP");
    }
    
    #[test]
    fn test_category_unification() {
        // Test unification of categories
        let mut feat1 = FeatureStructure::new();
        feat1.add("num", FeatureValue::Atomic("sg".to_string()));
        
        let mut feat2 = FeatureStructure::new();
        feat2.add("per", FeatureValue::Atomic("3".to_string()));
        
        let cat1 = CCGCategory::atomic_with_features("NP", feat1);
        let cat2 = CCGCategory::atomic_with_features("NP", feat2);
        
        // Compatible features should unify
        let unified = cat1.unify(&cat2);
        assert!(unified.is_some());
        
        // Different types should not unify
        let mut feat3 = FeatureStructure::new();
        feat3.add("num", FeatureValue::Atomic("sg".to_string()));
        let cat3 = CCGCategory::atomic_with_features("S", feat3);
        let unified2 = cat1.unify(&cat3);
        assert!(unified2.is_none());
    }
    #[test]
    fn test_complex_category_unification() {
        // Test unification of complex categories
        let np = CCGCategory::np();
        let n = CCGCategory::n();
        
        let complex1 = CCGCategory::forward(np.clone(), n.clone());
        let complex2 = CCGCategory::forward(np.clone(), n.clone());
        
        // Same complex categories should unify
        let unified = complex1.unify(&complex2);
        assert!(unified.is_some());
        
        // Different directions should not unify
        let complex3 = CCGCategory::backward(np.clone(), n.clone());
        let unified2 = complex1.unify(&complex3);
        assert!(unified2.is_none());
    }
}