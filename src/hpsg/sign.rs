//! Signs for Head-Driven Phrase Structure Grammar
//!
//! A sign is the basic linguistic object in HPSG, which can represent
//! words, phrases, and other linguistic entities. It consists of a
//! feature structure with specific features like PHON, SYNSEM, etc.

use std::fmt;
use crate::common::FeatureStructure as CommonFeatureStructure;
use crate::hpsg::feature_structure::{FeatureStructure, TypedValue, FeatureType};

/// A syntactic category in HPSG, simplified for compatibility with common interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Category {
    /// Name of the category (e.g., "noun", "verb")
    pub name: String,
    /// Morphosyntactic features (for compatibility with common interface)
    pub common_features: CommonFeatureStructure,
    /// The full HPSG feature structure
    pub feature_structure: Option<FeatureStructure>,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        
        if let Some(fs) = &self.feature_structure {
            write!(f, " {}", fs)?;
        } else if !self.common_features.features.is_empty() {
            write!(f, " {}", self.common_features)?;
        }
        
        Ok(())
    }
}

impl Category {
    /// Create a new category with just a name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            common_features: CommonFeatureStructure::new(),
            feature_structure: None,
        }
    }
    
    /// Create a category with common features
    pub fn with_features(name: &str, features: CommonFeatureStructure) -> Self {
        Self {
            name: name.to_string(),
            common_features: features,
            feature_structure: None,
        }
    }
    
    /// Create a category with an HPSG feature structure
    pub fn with_feature_structure(name: &str, fs: FeatureStructure) -> Self {
        Self {
            name: name.to_string(),
            common_features: fs.to_common(),
            feature_structure: Some(fs),
        }
    }
    
    /// Unify this category with another
    pub fn unify(&self, other: &Self) -> Option<Self> {
        // Categories must have the same name to unify
        if self.name != other.name {
            return None;
        }
        
        // If we have HPSG feature structures, use those for unification
        if let (Some(fs1), Some(fs2)) = (&self.feature_structure, &other.feature_structure) {
            if let Some(unified_fs) = fs1.unify(fs2) {
                return Some(Self::with_feature_structure(&self.name, unified_fs));
            }
            return None;
        }
        
        // Otherwise, fall back to common features
        if let Some(unified_features) = self.common_features.unify(&other.common_features) {
            return Some(Self::with_features(&self.name, unified_features));
        }
        
        None
    }
}

/// A Sign in HPSG (word, phrase, etc.)
#[derive(Debug, Clone)]
pub struct Sign {
    /// The type of this sign (e.g., "word", "phrase")
    pub sign_type: String,
    /// The feature structure for this sign
    pub feature_structure: FeatureStructure,
    /// The syntactic category (for compatibility)
    pub category: Category,
    /// The daughters (for phrases)
    pub daughters: Vec<Sign>,
    /// Index for identification
    pub index: usize,
}

impl Sign {
    /// Create a new sign with the given type and feature structure
    pub fn new(sign_type: &str, feature_structure: &FeatureStructure, index: usize) -> Self {
        // Extract the head type for the category
        let category_name = if let Some(head) = extract_head_type(feature_structure) {
            head
        } else {
            sign_type.to_string()
        };
        
        Self {
            sign_type: sign_type.to_string(),
            category: Category::with_feature_structure(&category_name, feature_structure.clone()), // Clone here
            feature_structure: feature_structure.clone(), // Clone here
            daughters: Vec::new(),
            index,
        }
    }
    
    /// Create a sign with daughters
    pub fn with_daughters(sign_type: &str, feature_structure: &FeatureStructure, daughters: Vec<Sign>, index: usize) -> Self {
        let mut sign = Self::new(sign_type, feature_structure, index);
        sign.daughters = daughters;
        sign
    }
    
    /// Create a lexical sign (word)
    pub fn lexical(phonetic_form: &str, mut feature_structure: FeatureStructure, index: usize) -> Self {
        // Set the PHON value
        let id = feature_structure.get_next_id();
        feature_structure.set("PHON", TypedValue {
            type_name: "string".to_string(),
            value: FeatureType::String(phonetic_form.to_string()),
            id,
        });
        
        Self::new("word", &feature_structure, index)
    }
    
    /// Create a sign for a phrase
    pub fn phrasal(head_daughter: Sign, non_head_daughters: Vec<Sign>, feature_structure: FeatureStructure, index: usize) -> Self {
        let mut daughters = vec![head_daughter];
        daughters.extend(non_head_daughters);
        
        Self::with_daughters("phrase", &feature_structure, daughters, index)
    }
    
    /// Find the head daughter
    pub fn head_daughter(&self) -> Option<&Sign> {
        if self.daughters.is_empty() {
            None
        } else {
            // In a more complete implementation, we would look at the HEAD-DTR feature
            // For simplicity, assume the first daughter is the head
            Some(&self.daughters[0])
        }
    }
    
    /// Get the phonetic form
    pub fn phonetic_form(&self) -> String {
        if let Some(phon_value) = self.feature_structure.get("PHON") {
            match &phon_value.value {
                FeatureType::String(s) => s.clone(),
                FeatureType::List(list) => {
                    let mut result = String::new();
                    for (i, item) in list.iter().enumerate() {
                        if i > 0 {
                            result.push(' ');
                        }
                        match &item.value {
                            FeatureType::String(s) => result.push_str(s),
                            _ => {}
                        }
                    }
                    result
                },
                _ => String::new(),
            }
        } else {
            // If no PHON feature, concatenate daughters
            let mut result = String::new();
            for (i, daughter) in self.daughters.iter().enumerate() {
                if i > 0 {
                    result.push(' ');
                }
                result.push_str(&daughter.phonetic_form());
            }
            result
        }
    }
    
    /// Unify this sign with another
    pub fn unify(&self, other: &Sign) -> Option<Sign> {
        // Signs must have compatible types
        if self.sign_type != other.sign_type {
            return None;
        }
        
        // Unify feature structures
        let unified_fs = self.feature_structure.unify(&other.feature_structure)?;
        
        // Create new sign with unified feature structure
        Some(Sign::new(&self.sign_type, &unified_fs, self.index.max(other.index) + 1))
    }
}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}] {}", self.sign_type, self.index, self.feature_structure)?;
        
        if !self.daughters.is_empty() {
            writeln!(f, "\nDaughters:")?;
            for (i, daughter) in self.daughters.iter().enumerate() {
                writeln!(f, "  {}: {}", i + 1, daughter)?;
            }
        }
        
        Ok(())
    }
}

/// Helper function to extract the head type from a feature structure
fn extract_head_type(fs: &FeatureStructure) -> Option<String> {
    if let Some(synsem) = fs.get("SYNSEM") {
        match &synsem.value {
            FeatureType::Reference(_ref_id) => {
                // In a real implementation, we would follow the reference
                None
            },
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_feature_structure(id: usize) -> FeatureStructure {
        let mut fs = FeatureStructure::new("phrase", id);
        
        fs.set("HEAD", TypedValue {
            type_name: "noun".to_string(),
            value: FeatureType::String("noun".to_string()),
            id: id + 1,
        });
        
        fs.set("PHON", TypedValue {
            type_name: "string".to_string(),
            value: FeatureType::String("test".to_string()),
            id: id + 2,
        });
        
        fs
    }
    
    #[test]
    fn test_category_creation() {
        let category = Category::new("noun");
        
        assert_eq!(category.name, "noun");
        assert!(category.common_features.features.is_empty());
        assert!(category.feature_structure.is_none());
    }
    
    #[test]
    fn test_category_with_features() {
        let mut features = CommonFeatureStructure::new();
        features.add("NUM", crate::common::FeatureValue::Atomic("sg".to_string()));
        
        let category = Category::with_features("noun", features);
        
        assert_eq!(category.name, "noun");
        assert!(!category.common_features.features.is_empty());
        assert!(category.feature_structure.is_none());
    }
    
    #[test]
    fn test_category_with_feature_structure() {
        let fs = create_test_feature_structure(1);
        
        let category = Category::with_feature_structure("noun", fs);
        
        assert_eq!(category.name, "noun");
        assert!(!category.common_features.features.is_empty());
        assert!(category.feature_structure.is_some());
    }
    
    #[test]
    fn test_category_unification() {
        let fs1 = create_test_feature_structure(1);
        let fs2 = create_test_feature_structure(3);
        
        let cat1 = Category::with_feature_structure("noun", fs1);
        let cat2 = Category::with_feature_structure("noun", fs2);
        
        let unified = cat1.unify(&cat2);
        assert!(unified.is_some());
        
        // Different categories should not unify
        let cat3 = Category::new("verb");
        let unified = cat1.unify(&cat3);
        assert!(unified.is_none());
    }
    
    #[test]
    fn test_sign_creation() {
        let fs = create_test_feature_structure(1);
        
        let sign = Sign::new("phrase", &fs, 1);
        
        assert_eq!(sign.sign_type, "phrase");
        assert_eq!(sign.feature_structure, fs);
        assert!(sign.daughters.is_empty());
        assert_eq!(sign.index, 1);
    }
    
    #[test]
    fn test_lexical_sign() {
        let fs = create_test_feature_structure(1);
        
        let sign = Sign::lexical("cat", fs, 1);
        
        assert_eq!(sign.sign_type, "word");
        assert_eq!(sign.phonetic_form(), "cat");
    }
    
    #[test]
    fn test_phrasal_sign() {
        let fs1 = create_test_feature_structure(1);
        let fs2 = create_test_feature_structure(3);
        
        let head = Sign::lexical("the", fs1.clone(), 1);
        let non_head = Sign::lexical("cat", fs2.clone(), 2);
        
        let phrase = Sign::phrasal(head, vec![non_head], fs1, 3);
        
        assert_eq!(phrase.sign_type, "phrase");
        assert_eq!(phrase.daughters.len(), 2);
        assert_eq!(phrase.phonetic_form(), "the cat");
    }
    
    #[test]
    fn test_sign_unification() {
        let fs1 = create_test_feature_structure(1);
        let fs2 = create_test_feature_structure(3);
        
        let sign1 = Sign::new("phrase", &fs1, 1);
        let sign2 = Sign::new("phrase", &fs2, 2);
        
        let unified = sign1.unify(&sign2);
        assert!(unified.is_some());
        
        // Different sign types should not unify
        let sign3 = Sign::new("word", &fs1, 3);
        let unified = sign1.unify(&sign3);
        assert!(unified.is_none());
    }
}