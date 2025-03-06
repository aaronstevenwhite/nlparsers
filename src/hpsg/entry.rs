//! Lexical entries for Head-Driven Phrase Structure Grammar
//!
//! This module provides lexical entries, which map words to their
//! lexical types and feature structures.

use std::fmt;
use crate::hpsg::feature_structure::{FeatureStructure, TypedValue, FeatureType};
use crate::hpsg::sign::{Sign, Category};

/// A lexical entry in HPSG
#[derive(Debug, Clone)]
pub struct LexicalEntry {
    /// The word form
    pub word: String,
    /// The sign type (usually "word")
    pub sign_type: String,
    /// The feature structure
    pub feature_structure: FeatureStructure,
}

impl LexicalEntry {
    /// Create a new lexical entry
    pub fn new(word: &str, feature_structure: FeatureStructure) -> Self {
        Self {
            word: word.to_string(),
            sign_type: "word".to_string(),
            feature_structure,
        }
    }
    
    /// Create a lexical entry with a specific sign type
    pub fn with_sign_type(word: &str, sign_type: &str, feature_structure: FeatureStructure) -> Self {
        Self {
            word: word.to_string(),
            sign_type: sign_type.to_string(),
            feature_structure,
        }
    }
    
    /// Create a lexical entry from a category (for compatibility)
    pub fn from_category(word: &str, category: Category, id: usize) -> Self {
        if let Some(fs) = category.feature_structure {
            Self::new(word, fs)
        } else {
            // Create a minimal feature structure from the category
            let mut fs = FeatureStructure::new(&category.name, id);
            
            // Set the HEAD feature
            fs.set("HEAD", TypedValue {
                type_name: category.name.clone(),
                value: FeatureType::String(category.name.clone()),
                id: id + 1,
            });
            
            Self::new(word, fs)
        }
    }
    
    /// Set a feature value
    pub fn set_feature(&mut self, path: &str, value: TypedValue) {
        // This is a simplified implementation that only handles top-level features
        // A full implementation would handle paths like SYNSEM.LOCAL.CAT.HEAD
        self.feature_structure.set(path, value);
    }
    
    /// Create a Sign from this lexical entry
    pub fn to_sign(&self, index: usize) -> Sign {
        Sign::new(
            &self.sign_type,
            &self.feature_structure,
            index
        )
    }
}

impl fmt::Display for LexicalEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]: {}", self.word, self.sign_type, self.feature_structure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::FeatureStructure as CommonFeatureStructure;
    use crate::common::FeatureValue as CommonFeatureValue;
    
    fn create_test_feature_structure() -> FeatureStructure {
        let mut fs = FeatureStructure::new("noun", 1);
        
        fs.set("HEAD", TypedValue {
            type_name: "noun".to_string(),
            value: FeatureType::String("noun".to_string()),
            id: 2,
        });
        
        fs
    }
    
    #[test]
    fn test_entry_creation() {
        let fs = create_test_feature_structure();
        let entry = LexicalEntry::new("cat", fs.clone());
        
        assert_eq!(entry.word, "cat");
        assert_eq!(entry.sign_type, "word");
        assert_eq!(entry.feature_structure, fs);
    }
    
    #[test]
    fn test_entry_with_sign_type() {
        let fs = create_test_feature_structure();
        let entry = LexicalEntry::with_sign_type("cat", "noun", fs.clone());
        
        assert_eq!(entry.word, "cat");
        assert_eq!(entry.sign_type, "noun");
        assert_eq!(entry.feature_structure, fs);
    }
    
    #[test]
    fn test_from_category() {
        let mut common_fs = CommonFeatureStructure::new();
        common_fs.add("NUM", CommonFeatureValue::Atomic("sg".to_string()));
        
        let category = Category::with_features("noun", common_fs);
        
        let entry = LexicalEntry::from_category("cat", category, 1);
        
        assert_eq!(entry.word, "cat");
        assert!(entry.feature_structure.has_feature("HEAD"));
    }
    
    #[test]
    fn test_set_feature() {
        let fs = create_test_feature_structure();
        let mut entry = LexicalEntry::new("cat", fs);
        
        entry.set_feature("NUM", TypedValue {
            type_name: "number".to_string(),
            value: FeatureType::String("sg".to_string()),
            id: 3,
        });
        
        assert!(entry.feature_structure.has_feature("NUM"));
        
        let num = entry.feature_structure.get("NUM").unwrap();
        assert_eq!(num.type_name, "number");
        
        match &num.value {
            FeatureType::String(s) => assert_eq!(s, "sg"),
            _ => panic!("Expected string value"),
        }
    }
    
    #[test]
    fn test_to_sign() {
        let fs = create_test_feature_structure();
        let entry = LexicalEntry::new("cat", fs);
        
        let sign = entry.to_sign(1);
        
        assert_eq!(sign.sign_type, "word");
        assert_eq!(sign.index, 1);
        assert_eq!(sign.phonetic_form(), "cat");
    }
    
    #[test]
    fn test_display() {
        let fs = create_test_feature_structure();
        let entry = LexicalEntry::new("cat", fs);
        
        let display = format!("{}", entry);
        assert!(display.contains("cat"));
        assert!(display.contains("word"));
        assert!(display.contains("noun"));
    }
}