//! Lexicon for Head-Driven Phrase Structure Grammar
//!
//! This module provides a lexicon that maps words to their
//! lexical entries in HPSG.

use std::collections::HashMap;
use crate::hpsg::feature_structure::{FeatureStructure, TypedValue, FeatureType};
use crate::hpsg::entry::LexicalEntry;
use crate::hpsg::sign::Category;

/// A lexicon mapping words to lexical entries
#[derive(Debug, Clone)]
pub struct Lexicon {
    /// Map from words to their possible lexical entries
    entries: HashMap<String, Vec<LexicalEntry>>,
    /// Counter for generating unique IDs
    next_id: usize,
}

impl Lexicon {
    /// Create a new empty lexicon
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_id: 0,
        }
    }
    
    /// Generate a unique ID for feature structures
    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Add a lexical entry
    pub fn add_entry(&mut self, word: &str, feature_structure: FeatureStructure) {
        let entry = LexicalEntry::new(word, feature_structure);
        
        self.entries
            .entry(word.to_string())
            .or_insert_with(Vec::new)
            .push(entry);
    }
    
    /// Add a lexical entry with a specific sign type
    pub fn add_entry_with_sign_type(&mut self, word: &str, sign_type: &str, feature_structure: FeatureStructure) {
        let entry = LexicalEntry::with_sign_type(word, sign_type, feature_structure);
        
        self.entries
            .entry(word.to_string())
            .or_insert_with(Vec::new)
            .push(entry);
    }
    
    /// Add a lexical entry from a category (for compatibility)
    pub fn add_from_category(&mut self, word: &str, category: Category) {
        let entry = LexicalEntry::from_category(word, category, self.next_id());
        
        self.entries
            .entry(word.to_string())
            .or_insert_with(Vec::new)
            .push(entry);
    }

    /// Get all possible lexical entries for a word
    pub fn get_entries(&self, word: &str) -> Vec<LexicalEntry> {
        match self.entries.get(word) {
            Some(entries) => entries.clone(),
            None => vec![],
        }
    }
    
    /// Check if a word is in the lexicon
    pub fn contains(&self, word: &str) -> bool {
        self.entries.contains_key(word)
    }
    
    /// Get the number of words in the lexicon
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if the lexicon is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Get all words in the lexicon
    pub fn get_words(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
    
    /// Create a basic lexicon with common entries
    pub fn basic() -> Self {
        let lexicon = Self::new();
        // No language-specific entries here
        lexicon
    }
}

impl Default for Lexicon {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_feature_structure(id: usize) -> FeatureStructure {
        let mut fs = FeatureStructure::new("noun", id);
        
        fs.set("HEAD", TypedValue {
            type_name: "noun".to_string(),
            value: FeatureType::String("noun".to_string()),
            id: id + 1,
        });
        
        fs
    }
    
    #[test]
    fn test_lexicon_creation() {
        let lexicon = Lexicon::new();
        
        assert!(lexicon.is_empty());
        assert_eq!(lexicon.len(), 0);
        assert_eq!(lexicon.next_id, 0);
    }
    
    #[test]
    fn test_add_entry() {
        let mut lexicon = Lexicon::new();
        let fs = create_test_feature_structure(lexicon.next_id());
        
        lexicon.add_entry("cat", fs);
        
        assert!(!lexicon.is_empty());
        assert_eq!(lexicon.len(), 1);
        assert!(lexicon.contains("cat"));
        assert!(!lexicon.contains("dog"));
    }
    
    #[test]
    fn test_add_entry_with_sign_type() {
        let mut lexicon = Lexicon::new();
        let fs = create_test_feature_structure(lexicon.next_id());
        
        lexicon.add_entry_with_sign_type("cat", "noun", fs);
        
        assert!(lexicon.contains("cat"));
        
        let entries = lexicon.get_entries("cat");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].sign_type, "noun");
    }
    
    #[test]
    fn test_add_from_category() {
        let mut lexicon = Lexicon::new();
        
        let category = Category::new("noun");
        lexicon.add_from_category("cat", category);
        
        assert!(lexicon.contains("cat"));
        
        let entries = lexicon.get_entries("cat");
        assert_eq!(entries.len(), 1);
    }
    
    #[test]
    fn test_get_entries() {
        let mut lexicon = Lexicon::new();
        let fs1 = create_test_feature_structure(lexicon.next_id());
        let fs2 = create_test_feature_structure(lexicon.next_id());
        
        lexicon.add_entry("bank", fs1);
        lexicon.add_entry_with_sign_type("bank", "verb", fs2);
        
        let entries = lexicon.get_entries("bank");
        assert_eq!(entries.len(), 2);
        
        let nonexistent = lexicon.get_entries("nonexistent");
        assert!(nonexistent.is_empty());
    }
    
    #[test]
    fn test_get_words() {
        let mut lexicon = Lexicon::new();
        let fs = create_test_feature_structure(lexicon.next_id());
        
        lexicon.add_entry("cat", fs.clone());
        lexicon.add_entry("dog", fs.clone());
        
        let words = lexicon.get_words();
        assert_eq!(words.len(), 2);
        assert!(words.contains(&"cat".to_string()));
        assert!(words.contains(&"dog".to_string()));
    }
    
    // Add a helper function to create an English lexicon for tests
    fn create_english_test_lexicon() -> Lexicon {
        let mut lexicon = Lexicon::new();
        
        // Helper function to create a noun feature structure
        let create_noun_fs = |id: usize| {
            let mut fs = FeatureStructure::new("noun", id);
            
            fs.set("HEAD", TypedValue {
                type_name: "noun".to_string(),
                value: FeatureType::String("noun".to_string()),
                id: id + 1,
            });
            
            fs
        };
        
        // Helper function to create a verb feature structure
        let create_verb_fs = |id: usize| {
            let mut fs = FeatureStructure::new("verb", id);
            
            fs.set("HEAD", TypedValue {
                type_name: "verb".to_string(),
                value: FeatureType::String("verb".to_string()),
                id: id + 1,
            });
            
            fs
        };
        
        // Helper function to create a determiner feature structure
        let create_det_fs = |id: usize| {
            let mut fs = FeatureStructure::new("det", id);
            
            fs.set("HEAD", TypedValue {
                type_name: "det".to_string(),
                value: FeatureType::String("det".to_string()),
                id: id + 1,
            });
            
            fs
        };
        
        // Add some basic English words for testing
        let id1 = lexicon.next_id();
        lexicon.add_entry("cat", create_noun_fs(id1));
        
        let id2 = lexicon.next_id();
        lexicon.add_entry("dog", create_noun_fs(id2));
        
        let id3 = lexicon.next_id();
        lexicon.add_entry("the", create_det_fs(id3));
        
        let id4 = lexicon.next_id();
        lexicon.add_entry("sleeps", create_verb_fs(id4));
        
        lexicon
    }
    
    #[test]
    fn test_english_lexicon() {
        let lexicon = create_english_test_lexicon();
        
        assert!(!lexicon.is_empty());
        assert!(lexicon.contains("cat"));
        assert!(lexicon.contains("dog"));
        assert!(lexicon.contains("the"));
        assert!(lexicon.contains("sleeps"));
    }
    
    #[test]
    fn test_next_id() {
        let mut lexicon = Lexicon::new();
        
        assert_eq!(lexicon.next_id(), 0);
        assert_eq!(lexicon.next_id(), 1);
        assert_eq!(lexicon.next_id(), 2);
    }
}