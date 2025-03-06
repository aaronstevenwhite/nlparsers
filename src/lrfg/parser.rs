//! Parser for Lexical-Realizational Functional Grammar
//!
//! This module provides a parser for LRFG that extends the LFG parser
//! with realizational capabilities.

use crate::lfg::{
    CStructure, Category,
    FStructure, Rule
};
use crate::common::Parser; // Import the Parser trait
use crate::lrfg::{
    r_structure::RStructure,
    mapping::{FRMapping, MappingRule},
    vocabulary::{Vocabulary, VocabularyItem}
};

/// Configuration for the LRFG parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Base LFG parser configuration
    pub lfg_config: crate::lfg::parser::ParserConfig,
    /// Whether to apply vocabulary items
    pub apply_vocabulary: bool,
    /// Debug mode
    pub debug: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            lfg_config: crate::lfg::parser::ParserConfig::default(),
            apply_vocabulary: true,
            debug: false,
        }
    }
}

/// Parser for Lexical-Realizational Functional Grammar
pub struct LRFGParser {
    /// Base LFG parser
    pub lfg_parser: crate::lfg::parser::LFGParser,
    /// F-structure to R-structure mapping
    pub mapping: FRMapping,
    /// Vocabulary
    pub vocabulary: Vocabulary,
    /// Configuration
    pub config: ParserConfig,
}

impl LRFGParser {
    /// Create a new LRFG parser
    pub fn new() -> Self {
        Self {
            lfg_parser: crate::lfg::parser::LFGParser::new(),
            mapping: FRMapping::new(),
            vocabulary: Vocabulary::new(),
            config: ParserConfig::default(),
        }
    }
    
    /// Create a new parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        Self {
            lfg_parser: crate::lfg::parser::LFGParser::new(),
            mapping: FRMapping::new(),
            vocabulary: Vocabulary::new(),
            config,
        }
    }
    
    /// Parse a sentence and generate its C-structure, F-structure, and R-structure
    pub fn parse(&self, sentence: &str) -> Option<LRFGParseResult> {
        // First, use the LFG parser to get C-structure and F-structure
        let lfg_result = self.lfg_parser.parse(sentence)?;
        
        // Create R-structure from F-structure
        let mut r_structure = self.mapping.apply(lfg_result.f_structure.as_ref()?);
        
        // Apply vocabulary items if configured
        if self.config.apply_vocabulary {
            self.vocabulary.realize_node(&mut r_structure.root);
        }
        
        // Debug output
        if self.config.debug {
            println!("=== LRFG Parse Result ===");
            println!("C-structure:");
            println!("{}", lfg_result);
            println!("F-structure:");
            println!("{}", lfg_result.f_structure.as_ref()?);
            println!("R-structure:");
            println!("{}", r_structure);
            println!("Realized form: {}", r_structure.realize());
        }
        Some(LRFGParseResult {
            c_structure: lfg_result.c_structure,
            r_structure: Some(r_structure),
        })
    }
    
    /// Generate a sentence from an F-structure
    pub fn generate(&self, f_structure: &FStructure) -> Option<String> {
        // Create R-structure from F-structure
        let mut r_structure = self.mapping.apply(f_structure);
        
        // Apply vocabulary items
        self.vocabulary.realize_node(&mut r_structure.root);
        
        // Realize the form
        let form = r_structure.realize();
        
        if form.is_empty() {
            None
        } else {
            Some(form)
        }
    }
    
    /// Add a mapping rule
    pub fn add_mapping_rule(&mut self, rule: MappingRule) {
        self.mapping.add_rule(rule);
    }
    
    /// Add a vocabulary item
    pub fn add_vocabulary_item(&mut self, item: VocabularyItem) {
        self.vocabulary.add_item(item);
    }
    
    /// Add a rule to the LFG parser
    pub fn add_rule(&mut self, rule: Rule) {
        self.lfg_parser.add_rule(rule);
    }
    
    /// Add a lexical entry to the LFG parser
    pub fn add_to_lexicon(&mut self, word: &str, category: Category) {
        self.lfg_parser.add_to_lexicon(word, category);
    }
}

/// Result of parsing with LRFG
#[derive(Debug, Clone)]
pub struct LRFGParseResult {
    /// C-structure from LFG parsing
    pub c_structure: CStructure,
    /// R-structure from mapping
    pub r_structure: Option<RStructure>,
}

impl LRFGParseResult {
    /// Get the realized form
    pub fn realize(&self) -> Option<String> {
        self.r_structure.as_ref().map(|r| r.realize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Setup English lexicon for testing
    fn setup_english_lexicon(parser: &mut LRFGParser) {
        // Add basic English words
        parser.add_to_lexicon("the", Category::new("Det"));
        parser.add_to_lexicon("a", Category::new("Det"));
        
        parser.add_to_lexicon("cat", Category::new("N"));
        parser.add_to_lexicon("dog", Category::new("N"));
        
        parser.add_to_lexicon("sleeps", Category::new("V"));
        parser.add_to_lexicon("sleep", Category::new("V"));
        parser.add_to_lexicon("sees", Category::new("V"));
        parser.add_to_lexicon("see", Category::new("V"));
    }
    
    /// Setup basic English grammar rules for testing
    fn setup_basic_english_grammar(parser: &mut LRFGParser) {
        // S -> NP VP
        parser.add_rule(Rule::new("S", vec!["NP", "VP"]));
        
        // NP -> Det N
        parser.add_rule(Rule::new("NP", vec!["Det", "N"]));
        
        // NP -> N
        parser.add_rule(Rule::new("NP", vec!["N"]));
        
        // VP -> V
        parser.add_rule(Rule::new("VP", vec!["V"]));
        
        // VP -> V NP
        parser.add_rule(Rule::new("VP", vec!["V", "NP"]));
    }
    
    /// Setup English mapping rules for testing
    fn setup_english_mapping(parser: &mut LRFGParser) {
        // Basic category mappings
        parser.add_mapping_rule(MappingRule::new("CAT", "cat", "NP"));
        parser.add_mapping_rule(MappingRule::new("SUBJ CAT", "cat", "NP"));
        parser.add_mapping_rule(MappingRule::new("OBJ CAT", "cat", "NP"));
        
        // Number and person features
        parser.add_mapping_rule(MappingRule::with_value("NUM", "sg", "num", "sg"));
        parser.add_mapping_rule(MappingRule::with_value("NUM", "pl", "num", "pl"));
        parser.add_mapping_rule(MappingRule::with_value("PERS", "1", "pers", "1"));
        parser.add_mapping_rule(MappingRule::with_value("PERS", "2", "pers", "2"));
        parser.add_mapping_rule(MappingRule::with_value("PERS", "3", "pers", "3"));
        
        // Tense
        parser.add_mapping_rule(MappingRule::with_value("TENSE", "pres", "tense", "pres"));
        parser.add_mapping_rule(MappingRule::with_value("TENSE", "past", "tense", "past"));
        
        // Determiners
        parser.add_mapping_rule(MappingRule::with_value("SPEC TYPE", "def", "cat", "Det"));
        parser.add_mapping_rule(MappingRule::with_value("SPEC TYPE", "indef", "cat", "Det"));
        
        // Subject and object mappings
        parser.add_mapping_rule(MappingRule::new("SUBJ NUM", "num", "sg"));
        parser.add_mapping_rule(MappingRule::new("SUBJ PERS", "pers", "3"));
        parser.add_mapping_rule(MappingRule::new("OBJ NUM", "num", "sg"));
        parser.add_mapping_rule(MappingRule::new("OBJ PERS", "pers", "3"));
    }
    
    /// Setup English grammar, mapping rules, and vocabulary for testing
    fn setup_english_grammar(parser: &mut LRFGParser) {
        // Setup lexicon
        setup_english_lexicon(parser);
        
        // Setup basic grammar rules
        setup_basic_english_grammar(parser);
        
        // Setup mapping rules
        setup_english_mapping(parser);
        
        // Setup vocabulary
        parser.vocabulary = Vocabulary::english();
    }
    
    /// Helper function to set up a basic parser for testing
    fn setup_test_parser() -> LRFGParser {
        let mut parser = LRFGParser::new();
        setup_english_grammar(&mut parser);
        parser
    }
    
    #[test]
    fn test_lrfg_parser() {
        let parser = setup_test_parser();
        
        // Parse a simple sentence
        let result = parser.parse("the cat sleeps");
        assert!(result.is_some());
        
        let result_clone = result.clone();
        let realized = result_clone.unwrap().realize();
        assert!(realized.is_some());
        assert_eq!(realized.unwrap(), "the cat sleeps");
    }
    
    #[test]
    fn test_generation() {
        let parser = setup_test_parser();
        
        // Create an F-structure for "the cat sleeps"
        let mut fs = FStructure::new(0);
        fs.set_pred("sleep", vec![]);
        fs.set("TENSE", crate::lfg::f_structure::FValue::Atomic("pres".to_string()));
        
        let mut subj = FStructure::new(1);
        subj.set_pred("cat", vec![]);
        subj.set("NUM", crate::lfg::f_structure::FValue::Atomic("sg".to_string()));
        subj.set("PERS", crate::lfg::f_structure::FValue::Atomic("3".to_string()));
        
        let mut spec = FStructure::new(2);
        spec.set("TYPE", crate::lfg::f_structure::FValue::Atomic("def".to_string()));
        
        subj.set("SPEC", crate::lfg::f_structure::FValue::Structure(Box::new(spec)));
        fs.set("SUBJ", crate::lfg::f_structure::FValue::Structure(Box::new(subj)));
        
        // Generate a sentence
        let generated = parser.generate(&fs);
        assert!(generated.is_some());
        
        // The exact form might vary depending on the vocabulary and mapping rules
        // but it should contain the key words
        let sentence = generated.unwrap();
        assert!(sentence.contains("the"));
        assert!(sentence.contains("cat"));
        assert!(sentence.contains("sleeps"));
    }
    
    #[test]
    fn test_parser_creation() {
        let parser = LRFGParser::new();
        
        assert!(parser.mapping.rules.is_empty());
        assert!(parser.vocabulary.items.is_empty());
    }
    
    #[test]
    fn test_with_config() {
        let mut config = ParserConfig::default();
        config.debug = true;
        config.apply_vocabulary = false;
        
        let parser = LRFGParser::with_config(config);
        
        assert!(parser.config.debug);
        assert!(!parser.config.apply_vocabulary);
    }
} 