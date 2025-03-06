//! Lexicon for Lexical-Functional Grammar
//!
//! This module provides a lexicon for LFG that maps words to lexical entries.

use std::collections::HashMap;
use crate::lfg::entry::LexicalEntry;
use crate::lfg::c_structure::Category;
use crate::lfg::f_structure::FConstraint;

/// Lexicon mapping words to lexical entries
#[derive(Debug, Clone)]
pub struct Lexicon {
    /// Map from words to their possible lexical entries
    entries: HashMap<String, Vec<LexicalEntry>>,
}

impl Lexicon {
    /// Create a new empty lexicon
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Add a lexical entry
    pub fn add(&mut self, entry: LexicalEntry) {
        self.entries
            .entry(entry.word.clone())
            .or_insert_with(Vec::new)
            .push(entry);
    }
    
    /// Add a simple word with just a category
    pub fn add_word(&mut self, word: &str, category: Category) {
        let entry = LexicalEntry::new(word, category);
        self.add(entry);
    }
    
    /// Add a word with predicate-argument structure
    pub fn add_pred(&mut self, word: &str, category: Category, pred: &str, args: Vec<&str>) {
        let entry = LexicalEntry::with_pred(word, category, pred, args);
        self.add(entry);
    }
    
    /// Add a word with constraints
    pub fn add_with_constraints(&mut self, word: &str, category: Category, constraints: Vec<FConstraint>) {
        let entry = LexicalEntry::with_constraints(word, category, constraints);
        self.add(entry);
    }
    
    /// Add a complete lexical entry
    pub fn add_complete(&mut self, word: &str, category: Category, pred: &str, args: Vec<&str>, constraints: Vec<FConstraint>) {
        let entry = LexicalEntry::complete(word, category, pred, args, constraints);
        self.add(entry);
    }

    /// Get all possible lexical entries for a word
    pub fn get_entries(&self, word: &str) -> Vec<LexicalEntry> {
        match self.entries.get(word) {
            Some(entries) => entries.clone(),
            None => vec![],
        }
    }
    
    /// Get all categories for a word
    pub fn get_categories(&self, word: &str) -> Vec<Category> {
        self.get_entries(word)
            .into_iter()
            .map(|entry| entry.category)
            .collect()
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
    
    /// Merge another lexicon into this one
    pub fn merge(&mut self, other: &Lexicon) {
        for (word, entries) in &other.entries {
            let target = self.entries.entry(word.clone()).or_insert_with(Vec::new);
            target.extend(entries.clone());
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
    fn test_lexicon_creation() {
        let mut lexicon = Lexicon::new();
        
        // Add some entries
        let n = Category::new("N");
        lexicon.add_word("cat", n.clone());
        
        assert!(lexicon.contains("cat"));
        assert_eq!(lexicon.len(), 1);
        
        // Add another entry for the same word
        let v = Category::new("V");
        lexicon.add_word("cat", v.clone());
        
        // Should still have length 1 (counting unique words)
        assert_eq!(lexicon.len(), 1);
        
        // But should have 2 entries for "cat"
        assert_eq!(lexicon.get_entries("cat").len(), 2);
        
        // Check categories
        let categories = lexicon.get_categories("cat");
        assert_eq!(categories.len(), 2);
        assert!(categories.iter().any(|c| c.name == "N"));
        assert!(categories.iter().any(|c| c.name == "V"));
    }
    
    #[test]
    fn test_english_lexicon() {
        let lexicon = setup_english_lexicon();
        
        // Check some common words
        assert!(lexicon.contains("cat"));
        assert!(lexicon.contains("dog"));
        assert!(lexicon.contains("the"));
        assert!(lexicon.contains("sleeps"));
        
        // Check a specific entry
        let entries = lexicon.get_entries("sleeps");
        assert_eq!(entries.len(), 1);
        
        let entry = &entries[0];
        assert_eq!(entry.category.name, "V");
        assert!(entry.pred.is_some());
        
        if let Some((pred, args)) = &entry.pred {
            assert_eq!(pred, "sleep");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], "SUBJ");
        }
        
        // Check constraints for agreement
        assert!(!entry.constraints.is_empty());
    }
    
    #[test]
    fn test_merge() {
        let mut lexicon1 = Lexicon::new();
        let mut lexicon2 = Lexicon::new();
        
        let n = Category::new("N");
        lexicon1.add_word("cat", n.clone());
        
        let v = Category::new("V");
        lexicon2.add_word("sleeps", v.clone());
        
        // Merge lexicon2 into lexicon1
        lexicon1.merge(&lexicon2);
        
        assert_eq!(lexicon1.len(), 2);
        assert!(lexicon1.contains("cat"));
        assert!(lexicon1.contains("sleeps"));
    }
    
    #[test]
    fn test_add_methods() {
        let mut lexicon = Lexicon::new();
        
        // Test add_word
        let n = Category::new("N");
        lexicon.add_word("cat", n.clone());
        
        // Test add_pred
        let v = Category::new("V");
        lexicon.add_pred("sleeps", v.clone(), "sleep", vec!["SUBJ"]);
        
        // Test add_with_constraints
        let det = Category::new("Det");
        lexicon.add_with_constraints("the", det.clone(), vec![
            FConstraint::Equation("↑DEF".to_string(), "yes".to_string()),
        ]);
        
        // Test add_complete
        let adj = Category::new("A");
        lexicon.add_complete("big", adj.clone(), "big", vec![], vec![
            FConstraint::Equation("↑SIZE".to_string(), "large".to_string()),
        ]);
        
        // Check results
        assert_eq!(lexicon.len(), 4);
        
        let cat_entries = lexicon.get_entries("cat");
        assert_eq!(cat_entries.len(), 1);
        assert_eq!(cat_entries[0].category.name, "N");
        assert!(cat_entries[0].pred.is_none());
        
        let sleeps_entries = lexicon.get_entries("sleeps");
        assert_eq!(sleeps_entries.len(), 1);
        assert_eq!(sleeps_entries[0].category.name, "V");
        assert!(sleeps_entries[0].pred.is_some());
        
        let the_entries = lexicon.get_entries("the");
        assert_eq!(the_entries.len(), 1);
        assert_eq!(the_entries[0].category.name, "Det");
        assert!(!the_entries[0].constraints.is_empty());
        
        let big_entries = lexicon.get_entries("big");
        assert_eq!(big_entries.len(), 1);
        assert_eq!(big_entries[0].category.name, "A");
        assert!(big_entries[0].pred.is_some());
        assert!(!big_entries[0].constraints.is_empty());
    }
    
    /// Helper function to create an English lexicon for testing
    pub fn setup_english_lexicon() -> Lexicon {
        let mut lexicon = Lexicon::new();
        
        // Function to add common constraints (prefixed with underscore to indicate intentional non-use)
        let _make_subj_constraint = || {
            vec![FConstraint::Equation("↑SUBJ NUM".to_string(), "↓NUM".to_string()),
                 FConstraint::Equation("↑SUBJ PERS".to_string(), "↓PERS".to_string())]
        };
        
        // Categories
        let n = Category::new("N");
        let v = Category::new("V");
        let det = Category::new("Det");
        let adj = Category::new("A");
        
        // Nouns
        lexicon.add_complete("cat", 
            n.clone(), 
            "cat", 
            vec![], 
            vec![FConstraint::Equation("↑NUM".to_string(), "sg".to_string()),
                 FConstraint::Equation("↑PERS".to_string(), "3".to_string())]
        );
        
        lexicon.add_complete("dog", 
            n.clone(), 
            "dog", 
            vec![], 
            vec![FConstraint::Equation("↑NUM".to_string(), "sg".to_string()),
                 FConstraint::Equation("↑PERS".to_string(), "3".to_string())]
        );
        
        lexicon.add_complete("cats", 
            n.clone(), 
            "cat", 
            vec![], 
            vec![FConstraint::Equation("↑NUM".to_string(), "pl".to_string()),
                 FConstraint::Equation("↑PERS".to_string(), "3".to_string())]
        );
        
        lexicon.add_complete("dogs", 
            n.clone(), 
            "dog", 
            vec![], 
            vec![FConstraint::Equation("↑NUM".to_string(), "pl".to_string()),
                 FConstraint::Equation("↑PERS".to_string(), "3".to_string())]
        );
        
        // Determiners
        lexicon.add_complete("the", 
            det.clone(), 
            "the", 
            vec![], 
            vec![]
        );
        
        lexicon.add_complete("a", 
            det.clone(), 
            "a", 
            vec![], 
            vec![FConstraint::ConstrainingEquation("↑NUM".to_string(), "sg".to_string())]
        );
        
        // Verbs
        lexicon.add_complete("sleeps", 
            v.clone(), 
            "sleep", 
            vec!["SUBJ"], 
            vec![FConstraint::Equation("↑TENSE".to_string(), "pres".to_string()),
                 FConstraint::ConstrainingEquation("↑SUBJ NUM".to_string(), "sg".to_string()),
                 FConstraint::ConstrainingEquation("↑SUBJ PERS".to_string(), "3".to_string())]
        );
        
        lexicon.add_complete("sleep", 
            v.clone(), 
            "sleep", 
            vec!["SUBJ"], 
            vec![FConstraint::Equation("↑TENSE".to_string(), "pres".to_string()),
                 FConstraint::ConstrainingEquation("↑SUBJ NUM".to_string(), "pl".to_string())]
        );
        
        lexicon.add_complete("sees", 
            v.clone(), 
            "see", 
            vec!["SUBJ", "OBJ"], 
            vec![FConstraint::Equation("↑TENSE".to_string(), "pres".to_string()),
                 FConstraint::ConstrainingEquation("↑SUBJ NUM".to_string(), "sg".to_string()),
                 FConstraint::ConstrainingEquation("↑SUBJ PERS".to_string(), "3".to_string())]
        );
        
        lexicon.add_complete("see", 
            v.clone(), 
            "see", 
            vec!["SUBJ", "OBJ"], 
            vec![FConstraint::Equation("↑TENSE".to_string(), "pres".to_string()),
                 FConstraint::ConstrainingEquation("↑SUBJ NUM".to_string(), "pl".to_string())]
        );
        
        // Adjectives
        lexicon.add_complete("big", 
            adj.clone(), 
            "big", 
            vec![], 
            vec![]
        );
        
        lexicon.add_complete("small", 
            adj.clone(), 
            "small", 
            vec![], 
            vec![]
        );
        
        lexicon
    }
    
    /// Helper function to create a comprehensive English lexicon for testing
    pub fn setup_comprehensive_english_lexicon() -> Lexicon {
        let mut lexicon = setup_english_lexicon();
        
        // Add more word categories
        let p = Category::new("P");      // Preposition
        let adv = Category::new("Adv");  // Adverb
        let conj = Category::new("Conj");     // Conjunction
        let comp = Category::new("C");   // Complementizer
        
        // Create WH-word category
        let wh = Category::new("WH");
        // Note: Since we can't directly modify features, we'll just use the category as is
        
        // Add prepositions
        lexicon.add_complete("in", 
            p.clone(), 
            "in", 
            vec!["OBJ"], 
            vec![FConstraint::Equation("↑PTYPE".to_string(), "loc".to_string())]
        );
        
        lexicon.add_complete("on", 
            p.clone(), 
            "on", 
            vec!["OBJ"], 
            vec![FConstraint::Equation("↑PTYPE".to_string(), "loc".to_string())]
        );
        
        lexicon.add_complete("with", 
            p.clone(), 
            "with", 
            vec!["OBJ"], 
            vec![FConstraint::Equation("↑PTYPE".to_string(), "comit".to_string())]
        );
        
        // Add adverbs
        lexicon.add_complete("quickly", 
            adv.clone(), 
            "quick", 
            vec![], 
            vec![FConstraint::Equation("↑MANNER".to_string(), "quick".to_string())]
        );
        
        lexicon.add_complete("slowly", 
            adv.clone(), 
            "slow", 
            vec![], 
            vec![FConstraint::Equation("↑MANNER".to_string(), "slow".to_string())]
        );
        
        // Add conjunctions
        lexicon.add_complete("and", 
            conj.clone(), 
            "and", 
            vec![], 
            vec![FConstraint::Equation("↑TYPE".to_string(), "coord".to_string())]
        );
        
        lexicon.add_complete("or", 
            conj.clone(), 
            "or", 
            vec![], 
            vec![FConstraint::Equation("↑TYPE".to_string(), "disj".to_string())]
        );
        
        // Add complementizers
        lexicon.add_complete("that", 
            comp.clone(), 
            "that", 
            vec![], 
            vec![FConstraint::Equation("↑TYPE".to_string(), "decl".to_string())]
        );
        
        // Add WH-words
        lexicon.add_complete("who", 
            wh.clone(), 
            "who", 
            vec![], 
            vec![
                FConstraint::Equation("↑ANIM".to_string(), "yes".to_string()),
                FConstraint::Equation("↑HUMAN".to_string(), "yes".to_string())
            ]
        );
        
        lexicon.add_complete("what", 
            wh.clone(), 
            "what", 
            vec![], 
            vec![
                FConstraint::Equation("↑ANIM".to_string(), "no".to_string())
            ]
        );
        
        // Add control verbs
        let v = Category::new("V");
        
        lexicon.add_complete("try", 
            v.clone(), 
            "try", 
            vec!["SUBJ", "XCOMP"], 
            vec![
                FConstraint::Equation("↑SUBJ".to_string(), "↑XCOMP SUBJ".to_string()),
                FConstraint::Equation("↑VTYPE".to_string(), "control".to_string())
            ]
        );
        
        lexicon.add_complete("want", 
            v.clone(), 
            "want", 
            vec!["SUBJ", "XCOMP"], 
            vec![
                FConstraint::Equation("↑SUBJ".to_string(), "↑XCOMP SUBJ".to_string()),
                FConstraint::Equation("↑VTYPE".to_string(), "control".to_string())
            ]
        );
        
        // Add raising verbs
        lexicon.add_complete("seem", 
            v.clone(), 
            "seem", 
            vec!["COMP"], 
            vec![
                FConstraint::Equation("↑SUBJ".to_string(), "↑COMP SUBJ".to_string()),
                FConstraint::Equation("↑VTYPE".to_string(), "raising".to_string())
            ]
        );
        
        lexicon.add_complete("appear", 
            v.clone(), 
            "appear", 
            vec!["COMP"], 
            vec![
                FConstraint::Equation("↑SUBJ".to_string(), "↑COMP SUBJ".to_string()),
                FConstraint::Equation("↑VTYPE".to_string(), "raising".to_string())
            ]
        );
        
        lexicon
    }
}