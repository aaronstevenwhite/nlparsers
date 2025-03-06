//! Lexical items in Minimalist Grammar

use std::fmt;
use crate::mg::feature::Feature;
use crate::common::{FeatureStructure, Category};

/// Item in the lexicon (lexical or functional)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LexicalItem {
    /// The phonological form
    pub phonetic_form: String,
    /// The feature bundle (head-initial)
    pub features: Vec<Feature>,
    /// Additional agreement information
    pub agreement_features: Option<FeatureStructure>,
}

impl fmt::Display for LexicalItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.phonetic_form)?;
        for (i, feature) in self.features.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", feature)?;
        }
        write!(f, "]")?;
        
        if let Some(agr) = &self.agreement_features {
            write!(f, "{}", agr)?;
        }
        
        Ok(())
    }
}

impl LexicalItem {
    /// Create a new lexical item with features
    pub fn new(pf: &str, features: Vec<Feature>) -> Self {
        LexicalItem {
            phonetic_form: pf.to_string(),
            features,
            agreement_features: None,
        }
    }
    
    /// Create a new lexical item with features and agreement information
    pub fn with_agreement(pf: &str, features: Vec<Feature>, agreement: FeatureStructure) -> Self {
        LexicalItem {
            phonetic_form: pf.to_string(),
            features,
            agreement_features: Some(agreement),
        }
    }
    
    /// Create a new empty lexical item (for traces)
    pub fn empty() -> Self {
        LexicalItem {
            phonetic_form: String::new(),
            features: Vec::new(),
            agreement_features: None,
        }
    }
    
    /// Check if this item has a particular feature type
    pub fn has_feature_type(&self, feature_type: &str) -> bool {
        self.features.iter().any(|f| match f {
            Feature::Categorial(s) => s == feature_type,
            Feature::Selector(s) => s == feature_type,
            Feature::Licensor(s) => s == feature_type,
            Feature::Licensee(s) => s == feature_type,
            Feature::StrongSelector(s) => s == feature_type,
            Feature::AdjunctSelector(s) => s == feature_type,
            Feature::Phase(s) => s == feature_type,
            Feature::Agreement(s, _) => s == feature_type,
            Feature::Delayed(inner) => match &**inner {
                Feature::Selector(s) => s == feature_type,
                _ => false,
            },
        })
    }
    
    /// Get the first feature, if any
    pub fn first_feature(&self) -> Option<&Feature> {
        self.features.first()
    }
    
    /// Remove the first feature from the feature bundle
    pub fn remove_first_feature(&mut self) -> Option<Feature> {
        if !self.features.is_empty() {
            Some(self.features.remove(0))
        } else {
            None
        }
    }
    
    /// Check if this item is a phase head
    pub fn is_phase_head(&self) -> bool {
        self.features.iter().any(|f| f.is_phase_head())
    }
    
    /// Check if this item is empty (for traces)
    pub fn is_empty(&self) -> bool {
        self.phonetic_form.is_empty() && self.features.is_empty()
    }
    
    /// Check if this item has delayed features for late merger
    pub fn has_delayed_features(&self) -> bool {
        self.features.iter().any(|f| f.is_delayed())
    }
    
    /// Get all delayed features
    pub fn get_delayed_features(&self) -> Vec<Feature> {
        self.features.iter()
            .filter_map(|f| {
                if let Feature::Delayed(inner) = f {
                    Some(*inner.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Create a copy with the first feature removed
    pub fn without_first_feature(&self) -> Self {
        let mut new_item = self.clone();
        new_item.remove_first_feature();
        new_item
    }
}

impl Category for LexicalItem {
    type Features = Vec<Feature>;
    
    fn features(&self) -> Option<&Self::Features> {
        Some(&self.features)
    }
    
    fn unify_with(&self, _other: &Self) -> Option<Self> {
        // For minimalist grammars, unification is typically not used
        // Return None to indicate that unification is not supported
        None
    }
    
    fn is_atomic(&self) -> bool {
        // A lexical item is atomic if it has exactly one feature (the categorial feature)
        self.features.len() == 1 && matches!(self.features.first(), Some(Feature::Categorial(_)))
    }
    
    fn atomic_name(&self) -> Option<&str> {
        // Return the categorial feature as the atomic name
        if let Some(Feature::Categorial(name)) = self.features.first() {
            Some(name.as_str())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{FeatureStructure, FeatureValue};
    
    #[test]
    fn test_lexical_item_creation() {
        // Test basic lexical item
        let features = vec![
            Feature::Categorial("D".to_string()),
            Feature::Licensee("case".to_string()),
        ];
        
        let item = LexicalItem::new("the", features);
        
        assert_eq!(item.phonetic_form, "the");
        assert_eq!(item.features.len(), 2);
        assert!(matches!(item.features[0], Feature::Categorial(_)));
        assert!(matches!(item.features[1], Feature::Licensee(_)));
        assert_eq!(item.agreement_features, None);
        
        // Test with agreement features
        let mut agr = FeatureStructure::new();
        agr.add("num", FeatureValue::Atomic("sg".to_string()));
        agr.add("per", FeatureValue::Atomic("3".to_string()));
        
        let item_with_agr = LexicalItem::with_agreement(
            "he", 
            vec![Feature::Categorial("D".to_string())],
            agr
        );
        
        assert_eq!(item_with_agr.phonetic_form, "he");
        assert!(item_with_agr.agreement_features.is_some());
        if let Some(agr) = &item_with_agr.agreement_features {
            assert_eq!(agr.get("num"), Some(&FeatureValue::Atomic("sg".to_string())));
            assert_eq!(agr.get("per"), Some(&FeatureValue::Atomic("3".to_string())));
        }
    }
    
    #[test]
    fn test_feature_operations() {
        let features = vec![
            Feature::Selector("D".to_string()),
            Feature::Categorial("T".to_string()),
            Feature::Phase("T".to_string()),
        ];
        
        let mut item = LexicalItem::new("T", features);
        
        // Test first feature
        if let Some(first) = item.first_feature() {
            assert!(matches!(first, Feature::Selector(_)));
        } else {
            panic!("Expected first feature");
        }
        
        // Test feature type checking
        assert!(item.has_feature_type("D"));
        assert!(item.has_feature_type("T"));
        assert!(!item.has_feature_type("C"));
        
        // Test phase head checking
        assert!(item.is_phase_head());
        
        // Test removing first feature
        if let Some(removed) = item.remove_first_feature() {
            assert!(matches!(removed, Feature::Selector(_)));
            assert_eq!(item.features.len(), 2);
            if let Some(new_first) = item.first_feature() {
                assert!(matches!(new_first, Feature::Categorial(_)));
            } else {
                panic!("Expected new first feature");
            }
        } else {
            panic!("Expected removed feature");
        }
    }
    
    #[test]
    fn test_delayed_features() {
        let features = vec![
            Feature::Delayed(Box::new(Feature::Selector("D".to_string()))),
            Feature::Categorial("N".to_string()),
        ];
        
        let item = LexicalItem::new("N", features);
        
        assert!(item.has_delayed_features());
        
        let delayed = item.get_delayed_features();
        assert_eq!(delayed.len(), 1);
        assert!(matches!(delayed[0], Feature::Selector(_)));
    }
    
    #[test]
    fn test_empty_item() {
        let empty = LexicalItem::empty();
        
        assert!(empty.is_empty());
        assert_eq!(empty.phonetic_form, "");
        assert_eq!(empty.features.len(), 0);
    }
    
    #[test]
    fn test_item_display() {
        let features = vec![
            Feature::Categorial("D".to_string()),
            Feature::Licensee("case".to_string()),
        ];
        
        let item = LexicalItem::new("the", features);
        
        assert_eq!(item.to_string(), "the[D -case]");
        
        // Test with agreement features
        let mut agr = FeatureStructure::new();
        agr.add("num", FeatureValue::Atomic("sg".to_string()));
        
        let item_with_agr = LexicalItem::with_agreement(
            "he", 
            vec![Feature::Categorial("D".to_string())],
            agr
        );
        
        assert_eq!(item_with_agr.to_string(), "he[D][num=sg]");
    }
}