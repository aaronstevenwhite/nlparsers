//! Generic lexicon implementation for any grammar formalism

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Generic lexicon that maps words to their possible categories in a grammar formalism
#[derive(Debug, Clone)]
pub struct Lexicon<C> 
where
    C: Clone + PartialEq + Eq + Hash
{
    /// Map from words to their possible categories
    entries: HashMap<String, HashSet<C>>,
}

impl<C> Lexicon<C> 
where
    C: Clone + PartialEq + Eq + Hash
{
    /// Create a new empty lexicon
    pub fn new() -> Self {
        Lexicon {
            entries: HashMap::new(),
        }
    }

    /// Add a word with its category to the lexicon
    pub fn add(&mut self, word: &str, category: C) {
        self.entries
            .entry(word.to_string())
            .or_insert_with(HashSet::new)
            .insert(category);
    }

    /// Get all possible categories for a word
    pub fn get_categories(&self, word: &str) -> Vec<C> {
        match self.entries.get(word) {
            Some(categories) => categories.iter().cloned().collect(),
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
    
    /// Remove a specific category for a word
    pub fn remove_category(&mut self, word: &str, category: &C) {
        if let Some(categories) = self.entries.get_mut(word) {
            categories.remove(category);
            if categories.is_empty() {
                self.entries.remove(word);
            }
        }
    }
    
    /// Clear the lexicon
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Get an iterator over all entries in the lexicon
    pub fn iter(&self) -> impl Iterator<Item = (&String, &HashSet<C>)> {
        self.entries.iter()
    }
    
    /// Check if a word has a specific category
    pub fn has_category(&self, word: &str, category: &C) -> bool {
        if let Some(categories) = self.entries.get(word) {
            categories.contains(category)
        } else {
            false
        }
    }
}

// Implement default for empty lexicon
impl<C> Default for Lexicon<C> 
where
    C: Clone + PartialEq + Eq + Hash
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // A simple category enum for testing
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum TestCategory {
        Noun,
        Verb,
        Adjective,
    }
    
    #[test]
    fn test_add_to_lexicon() {
        let mut lexicon = Lexicon::new();
        
        lexicon.add("cat", TestCategory::Noun);
        lexicon.add("run", TestCategory::Verb);
        
        assert!(lexicon.contains("cat"));
        assert!(lexicon.contains("run"));
        assert!(!lexicon.contains("dog"));
        
        assert_eq!(lexicon.len(), 2);
    }
    
    #[test]
    fn test_multiple_categories() {
        let mut lexicon = Lexicon::new();
        
        // "bank" can be a noun or a verb
        lexicon.add("bank", TestCategory::Noun);
        lexicon.add("bank", TestCategory::Verb);
        
        let categories = lexicon.get_categories("bank");
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&TestCategory::Noun));
        assert!(categories.contains(&TestCategory::Verb));
    }
    
    #[test]
    fn test_remove_from_lexicon() {
        let mut lexicon = Lexicon::new();
        
        lexicon.add("cat", TestCategory::Noun);
        lexicon.add("dog", TestCategory::Noun);
        
        assert!(lexicon.contains("cat"));
        lexicon.remove("cat");
        assert!(!lexicon.contains("cat"));
        assert!(lexicon.contains("dog"));
        
        assert_eq!(lexicon.len(), 1);
    }
    
    #[test]
    fn test_remove_category() {
        let mut lexicon = Lexicon::new();
        
        lexicon.add("bank", TestCategory::Noun);
        lexicon.add("bank", TestCategory::Verb);
        
        assert!(lexicon.has_category("bank", &TestCategory::Verb));
        lexicon.remove_category("bank", &TestCategory::Verb);
        assert!(!lexicon.has_category("bank", &TestCategory::Verb));
        assert!(lexicon.has_category("bank", &TestCategory::Noun));
        
        // Word should still be in the lexicon with one category
        assert!(lexicon.contains("bank"));
        assert_eq!(lexicon.get_categories("bank").len(), 1);
    }
}