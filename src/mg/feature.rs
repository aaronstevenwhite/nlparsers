//! Features in Minimalist Grammar

use std::fmt;

/// Features in Minimalist Grammar
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Feature {
    /// Categorial features (e.g., D, T, C, v)
    Categorial(String),
    /// Selector features (e.g., =D, =V)
    Selector(String),
    /// Licensor features (e.g., +wh, +case)
    Licensor(String),
    /// Licensee features (e.g., -wh, -case)
    Licensee(String),
    /// Strong selector features that trigger head movement (e.g., =v+)
    StrongSelector(String),
    /// Adjunct selector features (e.g., ~A, ~Adv)
    AdjunctSelector(String),
    /// Agreement features (e.g., φ:3sg)
    Agreement(String, String),
    /// Phase feature marking phase boundaries (e.g., ⚑C, ⚑v)
    Phase(String),
    /// Optionally delayed feature for late merger (e.g., =D[delay])
    Delayed(Box<Feature>),
}

impl fmt::Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Feature::Categorial(s) => write!(f, "{}", s),
            Feature::Selector(s) => write!(f, "={}", s),
            Feature::Licensor(s) => write!(f, "+{}", s),
            Feature::Licensee(s) => write!(f, "-{}", s),
            Feature::StrongSelector(s) => write!(f, "={}+", s),
            Feature::AdjunctSelector(s) => write!(f, "~{}", s),
            Feature::Agreement(key, val) => write!(f, "φ:{}={}", key, val),
            Feature::Phase(s) => write!(f, "⚑{}", s),
            Feature::Delayed(inner) => write!(f, "{}[delay]", inner),
        }
    }
}

impl Feature {
    /// Create a new categorial feature
    pub fn categorial(name: &str) -> Self {
        Feature::Categorial(name.to_string())
    }
    
    /// Create a new selector feature
    pub fn selector(name: &str) -> Self {
        Feature::Selector(name.to_string())
    }
    
    /// Create a new licensor feature
    pub fn licensor(name: &str) -> Self {
        Feature::Licensor(name.to_string())
    }
    
    /// Create a new licensee feature
    pub fn licensee(name: &str) -> Self {
        Feature::Licensee(name.to_string())
    }
    
    /// Create a new strong selector feature
    pub fn strong_selector(name: &str) -> Self {
        Feature::StrongSelector(name.to_string())
    }
    
    /// Create a new adjunct selector feature
    pub fn adjunct_selector(name: &str) -> Self {
        Feature::AdjunctSelector(name.to_string())
    }
    
    /// Create a new agreement feature
    pub fn agreement(key: &str, val: &str) -> Self {
        Feature::Agreement(key.to_string(), val.to_string())
    }
    
    /// Create a new phase feature
    pub fn phase(name: &str) -> Self {
        Feature::Phase(name.to_string())
    }
    
    /// Create a new delayed feature
    pub fn delayed(inner: Feature) -> Self {
        Feature::Delayed(Box::new(inner))
    }
    
    /// Check if this feature matches another for Merge operation
    pub fn matches(&self, other: &Feature) -> bool {
        match (self, other) {
            (Feature::Selector(s1), Feature::Categorial(s2)) => s1 == s2,
            (Feature::StrongSelector(s1), Feature::Categorial(s2)) => s1 == s2,
            _ => false,
        }
    }
    
    /// Check if this feature matches another for Move operation
    pub fn matches_move(&self, other: &Feature) -> bool {
        match (self, other) {
            (Feature::Licensor(s1), Feature::Licensee(s2)) => s1 == s2,
            _ => false,
        }
    }
    
    /// Check if this feature can trigger head movement
    pub fn triggers_head_movement(&self) -> bool {
        matches!(self, Feature::StrongSelector(_))
    }
    
    /// Check if this feature is a phase head
    pub fn is_phase_head(&self) -> bool {
        matches!(self, Feature::Phase(_))
    }
    
    /// Check if this feature is delayed for late merger
    pub fn is_delayed(&self) -> bool {
        matches!(self, Feature::Delayed(_))
    }
    
    /// Get the inner feature if this is a delayed feature
    pub fn get_delayed_feature(&self) -> Option<&Feature> {
        match self {
            Feature::Delayed(inner) => Some(inner),
            _ => None,
        }
    }
    
    /// Unwrap a delayed feature
    pub fn unwrap_delayed(&self) -> &Feature {
        match self {
            Feature::Delayed(inner) => inner,
            _ => self,
        }
    }
}

// Add this implementation to allow &str to be converted to FeatureValue
impl From<&str> for crate::common::FeatureValue {
    fn from(s: &str) -> Self {
        crate::common::FeatureValue::Atomic(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_display() {
        let cat = Feature::Categorial("D".to_string());
        let sel = Feature::Selector("N".to_string());
        let lic = Feature::Licensor("wh".to_string());
        let lee = Feature::Licensee("case".to_string());
        
        assert_eq!(cat.to_string(), "D");
        assert_eq!(sel.to_string(), "=N");
        assert_eq!(lic.to_string(), "+wh");
        assert_eq!(lee.to_string(), "-case");
    }
    
    #[test]
    fn test_feature_matching() {
        // Test matching features for Merge
        let sel = Feature::Selector("D".to_string());
        let cat = Feature::Categorial("D".to_string());
        let strong_sel = Feature::StrongSelector("D".to_string());
        
        assert!(sel.matches(&cat));
        assert!(strong_sel.matches(&cat));
        
        let diff_cat = Feature::Categorial("N".to_string());
        assert!(!sel.matches(&diff_cat));
        
        // Test matching features for Move
        let lic = Feature::Licensor("wh".to_string());
        let lee = Feature::Licensee("wh".to_string());
        
        assert!(lic.matches_move(&lee));
        
        let diff_lee = Feature::Licensee("case".to_string());
        assert!(!lic.matches_move(&diff_lee));
    }
    
    #[test]
    fn test_special_features() {
        let strong = Feature::StrongSelector("v".to_string());
        let phase = Feature::Phase("C".to_string());
        let delayed = Feature::Delayed(Box::new(Feature::Selector("D".to_string())));
        
        assert!(strong.triggers_head_movement());
        assert!(phase.is_phase_head());
        assert!(delayed.is_delayed());
        
        if let Some(inner) = delayed.get_delayed_feature() {
            assert!(matches!(inner, Feature::Selector(_)));
        } else {
            panic!("Expected Some(inner_feature)");
        }
    }
    
    #[test]
    fn test_feature_constructors() {
        let cat = Feature::categorial("D");
        let sel = Feature::selector("N");
        let lic = Feature::licensor("wh");
        let lee = Feature::licensee("case");
        let strong = Feature::strong_selector("v");
        let adj = Feature::adjunct_selector("A");
        let agr = Feature::agreement("num", "sg");
        let phase = Feature::phase("C");
        let delayed = Feature::delayed(Feature::selector("D"));
        
        assert_eq!(cat, Feature::Categorial("D".to_string()));
        assert_eq!(sel, Feature::Selector("N".to_string()));
        assert_eq!(lic, Feature::Licensor("wh".to_string()));
        assert_eq!(lee, Feature::Licensee("case".to_string()));
        assert_eq!(strong, Feature::StrongSelector("v".to_string()));
        assert_eq!(adj, Feature::AdjunctSelector("A".to_string()));
        assert_eq!(agr, Feature::Agreement("num".to_string(), "sg".to_string()));
        assert_eq!(phase, Feature::Phase("C".to_string()));
        assert_eq!(delayed, Feature::Delayed(Box::new(Feature::Selector("D".to_string()))));
    }
}