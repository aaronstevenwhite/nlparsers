//! Lexicon for Type-Logical Grammar

use std::fmt;
use std::collections::HashMap;
use crate::tlg::logical_type::LogicalType;

/// Lexical item in Type-Logical Grammar
#[derive(Debug, Clone)]
pub struct LexicalItem {
    /// The word form
    pub word: String,
    /// The logical type
    pub logical_type: LogicalType,
    /// Phonological form for prosodic interpretation
    pub phonological_form: Option<String>,
}

impl fmt::Display for LexicalItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(phon) = &self.phonological_form {
            write!(f, "{} ⊢ {} : {}", self.word, self.logical_type, phon)
        } else {
            write!(f, "{} ⊢ {}", self.word, self.logical_type)
        }
    }
}

impl LexicalItem {
    /// Create a new lexical item with just word and type
    pub fn new(word: &str, logical_type: LogicalType) -> Self {
        Self {
            word: word.to_string(),
            logical_type,
            phonological_form: None,
        }
    }
    
    /// Create a new lexical item with phonological form
    pub fn with_phonology(word: &str, logical_type: LogicalType, phon: &str) -> Self {
        Self {
            word: word.to_string(),
            logical_type,
            phonological_form: Some(phon.to_string()),
        }
    }
}

/// The lexicon maps words to their possible logical types
#[derive(Debug, Clone)]
pub struct Lexicon {
    entries: HashMap<String, Vec<LexicalItem>>,
}

impl Lexicon {
    /// Create a new empty lexicon
    pub fn new() -> Self {
        Lexicon {
            entries: HashMap::new(),
        }
    }

    /// Add a word with its logical type to the lexicon
    pub fn add(&mut self, word: &str, logical_type: LogicalType) {
        self.entries
            .entry(word.to_string())
            .or_insert_with(Vec::new)
            .push(LexicalItem::new(word, logical_type));
    }
    
    /// Add a word with its logical type and phonological form to the lexicon
    pub fn add_with_phonology(&mut self, word: &str, logical_type: LogicalType, phon: &str) {
        self.entries
            .entry(word.to_string())
            .or_insert_with(Vec::new)
            .push(LexicalItem::with_phonology(word, logical_type, phon));
    }

    /// Get all possible lexical items for a word
    pub fn get_items(&self, word: &str) -> Vec<LexicalItem> {
        match self.entries.get(word) {
            Some(items) => items.clone(),
            None => vec![],
        }
    }
    
    /// Get all possible logical types for a word
    pub fn get_types(&self, word: &str) -> Vec<LogicalType> {
        match self.entries.get(word) {
            Some(items) => items.iter().map(|item| item.logical_type.clone()).collect(),
            None => vec![],
        }
    }
    
    /// Check if a word is in the lexicon
    pub fn contains(&self, word: &str) -> bool {
        self.entries.contains_key(word)
    }
    
    /// Get the number of entries in the lexicon
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
    
    /// Remove a word from the lexicon
    pub fn remove(&mut self, word: &str) {
        self.entries.remove(word);
    }
    
    /// Add all entries from another lexicon
    pub fn merge(&mut self, other: &Lexicon) {
        for (word, items) in &other.entries {
            for item in items {
                self.add(word, item.logical_type.clone());
            }
        }
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
    
    #[test]
    fn test_lexical_item_creation() {
        let np = LogicalType::np();
        let s = LogicalType::s();
        
        let verb_type = LogicalType::left_impl(s.clone(), np.clone());
        
        let item = LexicalItem::new("sleeps", verb_type.clone());
        assert_eq!(item.word, "sleeps");
        assert_eq!(item.to_string(), "sleeps ⊢ s ← np");
        
        let item_with_phon = LexicalItem::with_phonology("sleeps", verb_type, "'sleeps'");
        assert_eq!(item_with_phon.to_string(), "sleeps ⊢ s ← np : 'sleeps'");
    }
    
    #[test]
    fn test_lexicon_operations() {
        let mut lexicon = Lexicon::new();
        let np = LogicalType::np();
        let s = LogicalType::s();
        
        // Intransitive verb type
        let iv_type = LogicalType::left_impl(s.clone(), np.clone());
        
        // Add some entries
        lexicon.add("sleeps", iv_type.clone());
        lexicon.add("runs", iv_type.clone());
        
        // Check basic operations
        assert!(lexicon.contains("sleeps"));
        assert!(lexicon.contains("runs"));
        assert!(!lexicon.contains("jumps"));
        
        assert_eq!(lexicon.len(), 2);
        
        // Get types
        let types = lexicon.get_types("sleeps");
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].to_string(), "s ← np");
        
        // Remove an entry
        lexicon.remove("sleeps");
        assert!(!lexicon.contains("sleeps"));
        assert_eq!(lexicon.len(), 1);
        
        // Test words list
        let words = lexicon.get_words();
        assert_eq!(words.len(), 1);
        assert!(words.contains(&"runs".to_string()));
    }
    
    #[test]
    fn test_lexicon_with_multiple_types() {
        let mut lexicon = Lexicon::new();
        let np = LogicalType::np();
        let n = LogicalType::n();
        
        // 'bank' can be a noun or verb
        lexicon.add("bank", n.clone());
        
        let iv_type = LogicalType::left_impl(LogicalType::s(), np.clone());
        lexicon.add("bank", iv_type.clone());
        
        // Should have two types for 'bank'
        let types = lexicon.get_types("bank");
        assert_eq!(types.len(), 2);
        
        // Get items should also return two entries
        let items = lexicon.get_items("bank");
        assert_eq!(items.len(), 2);
    }
    
    #[test]
    fn test_lexicon_merge() {
        let mut lexicon1 = Lexicon::new();
        let mut lexicon2 = Lexicon::new();
        
        lexicon1.add("cat", LogicalType::n());
        lexicon2.add("dog", LogicalType::n());
        
        lexicon1.merge(&lexicon2);
        
        assert!(lexicon1.contains("cat"));
        assert!(lexicon1.contains("dog"));
        assert_eq!(lexicon1.len(), 2);
    }
}