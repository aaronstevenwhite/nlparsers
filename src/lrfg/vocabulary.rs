//! Vocabulary items for Lexical-Realizational Functional Grammar
//!
//! This module provides vocabulary items that map R-structure features
//! to phonological forms.

use std::fmt;
use std::collections::{HashMap, HashSet};
use crate::lrfg::r_structure::{RNode, RFeature};

/// A vocabulary item in LRFG
#[derive(Debug, Clone)]
pub struct VocabularyItem {
    /// Features that this item realizes
    pub features: HashSet<RFeature>,
    /// Phonological form
    pub form: String,
    /// Priority (for competition)
    pub priority: usize,
}

impl VocabularyItem {
    /// Create a new vocabulary item
    pub fn new(form: &str) -> Self {
        Self {
            features: HashSet::new(),
            form: form.to_string(),
            priority: 0,
        }
    }
    
    /// Add a feature to this vocabulary item
    pub fn add_feature(&mut self, name: &str, value: &str) -> &mut Self {
        self.features.insert(RFeature::new(name, value));
        self
    }
    
    /// Set the priority of this vocabulary item
    pub fn with_priority(&mut self, priority: usize) -> &mut Self {
        self.priority = priority;
        self
    }
    
    /// Check if this item matches a set of features
    pub fn matches(&self, features: &HashSet<RFeature>) -> bool {
        // All features of this item must be present in the input features
        self.features.iter().all(|f| features.contains(f))
    }
    
    /// Count how many features match
    pub fn match_count(&self, features: &HashSet<RFeature>) -> usize {
        self.features.iter().filter(|f| features.contains(f)).count()
    }
}

impl fmt::Display for VocabularyItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\" ↔ ", self.form)?;
        
        let features: Vec<_> = self.features.iter().collect();
        for (i, feature) in features.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", feature)?;
        }
        
        if self.priority > 0 {
            write!(f, " (priority: {})", self.priority)?;
        }
        
        Ok(())
    }
}

/// A vocabulary of items
#[derive(Debug, Clone)]
pub struct Vocabulary {
    /// List of vocabulary items
    pub items: Vec<VocabularyItem>,
}

impl Vocabulary {
    /// Create a new empty vocabulary
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }
    
    /// Add a vocabulary item
    pub fn add_item(&mut self, item: VocabularyItem) {
        self.items.push(item);
    }
    
    /// Find the best matching item for a set of features
    pub fn find_match(&self, features: &HashSet<RFeature>) -> Option<&VocabularyItem> {
        let mut best_match = None;
        let mut best_count = 0;
        let mut best_priority = 0;
        
        for item in &self.items {
            if item.matches(features) {
                let count = item.match_count(features);
                
                // Check if this is a better match
                if best_match.is_none() || 
                   item.priority > best_priority ||
                   (item.priority == best_priority && count > best_count) {
                    best_match = Some(item);
                    best_count = count;
                    best_priority = item.priority;
                }
            }
        }
        
        best_match
    }
    
    /// Apply vocabulary items to an R-structure node
    pub fn realize_node(&self, node: &mut RNode) {
        // Find the best matching vocabulary item
        if let Some(item) = self.find_match(&node.features) {
            node.set_form(&item.form);
        }
        
        // Realize children
        for child in &mut node.children {
            self.realize_node(child);
        }
    }
    
    /// Create a basic English vocabulary
    pub fn english() -> Self {
        let mut vocab = Self::new();
        
        // Determiners
        let mut the = VocabularyItem::new("the");
        the.add_feature("cat", "Det");
        vocab.add_item(the);
        
        let mut a = VocabularyItem::new("a");
        a.add_feature("cat", "Det")
         .add_feature("num", "sg");
        vocab.add_item(a);
        
        // Nouns
        let mut cat_sg = VocabularyItem::new("cat");
        cat_sg.add_feature("pred", "cat")
             .add_feature("num", "sg");
        vocab.add_item(cat_sg);
        
        let mut cat_pl = VocabularyItem::new("cats");
        cat_pl.add_feature("pred", "cat")
             .add_feature("num", "pl");
        vocab.add_item(cat_pl);
        
        let mut dog_sg = VocabularyItem::new("dog");
        dog_sg.add_feature("pred", "dog")
             .add_feature("num", "sg");
        vocab.add_item(dog_sg);
        
        let mut dog_pl = VocabularyItem::new("dogs");
        dog_pl.add_feature("pred", "dog")
             .add_feature("num", "pl");
        vocab.add_item(dog_pl);
        
        // Verbs
        let mut sleep_3sg = VocabularyItem::new("sleeps");
        sleep_3sg.add_feature("pred", "sleep")
                .add_feature("num", "sg")
                .add_feature("pers", "3")
                .add_feature("tense", "pres");
        vocab.add_item(sleep_3sg);
        
        let mut sleep_other = VocabularyItem::new("sleep");
        sleep_other.add_feature("pred", "sleep")
                  .add_feature("tense", "pres");
        vocab.add_item(sleep_other);
        
        let mut see_3sg = VocabularyItem::new("sees");
        see_3sg.add_feature("pred", "see")
              .add_feature("num", "sg")
              .add_feature("pers", "3")
              .add_feature("tense", "pres");
        vocab.add_item(see_3sg);
        
        let mut see_other = VocabularyItem::new("see");
        see_other.add_feature("pred", "see")
                .add_feature("tense", "pres");
        vocab.add_item(see_other);
        
        // Past tense
        let mut slept = VocabularyItem::new("slept");
        slept.add_feature("pred", "sleep")
            .add_feature("tense", "past");
        vocab.add_item(slept);
        
        let mut saw = VocabularyItem::new("saw");
        saw.add_feature("pred", "see")
           .add_feature("tense", "past");
        vocab.add_item(saw);
        
        vocab
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vocabulary_item() {
        let mut item = VocabularyItem::new("cat");
        item.add_feature("pred", "cat")
            .add_feature("num", "sg");
        
        let mut features = HashSet::new();
        features.insert(RFeature::new("pred", "cat"));
        features.insert(RFeature::new("num", "sg"));
        features.insert(RFeature::new("pers", "3"));
        
        assert!(item.matches(&features));
        assert_eq!(item.match_count(&features), 2);
        
        // Test non-matching features
        let mut features2 = HashSet::new();
        features2.insert(RFeature::new("pred", "dog"));
        features2.insert(RFeature::new("num", "sg"));
        
        assert!(!item.matches(&features2));
    }
    
    #[test]
    fn test_vocabulary() {
        let vocab = Vocabulary::english();
        
        // Test matching for "cat"
        let mut features = HashSet::new();
        features.insert(RFeature::new("pred", "cat"));
        features.insert(RFeature::new("num", "sg"));
        
        let match_result = vocab.find_match(&features);
        assert!(match_result.is_some());
        assert_eq!(match_result.unwrap().form, "cat");
        
        // Test matching for "cats"
        let mut features2 = HashSet::new();
        features2.insert(RFeature::new("pred", "cat"));
        features2.insert(RFeature::new("num", "pl"));
        
        let match_result2 = vocab.find_match(&features2);
        assert!(match_result2.is_some());
        assert_eq!(match_result2.unwrap().form, "cats");
        
        // Test matching for "sleeps"
        let mut features3 = HashSet::new();
        features3.insert(RFeature::new("pred", "sleep"));
        features3.insert(RFeature::new("num", "sg"));
        features3.insert(RFeature::new("pers", "3"));
        features3.insert(RFeature::new("tense", "pres"));
        
        let match_result3 = vocab.find_match(&features3);
        assert!(match_result3.is_some());
        assert_eq!(match_result3.unwrap().form, "sleeps");
    }
    
    #[test]
    fn test_realize_node() {
        let vocab = Vocabulary::english();
        
        // Create an R-structure node
        let mut node = RNode::new(0);
        node.add_feature("pred", "cat");
        node.add_feature("num", "sg");
        
        // Realize the node
        vocab.realize_node(&mut node);
        
        assert_eq!(node.form, Some("cat".to_string()));
    }
} 