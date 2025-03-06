//! Parser for Lexical-Functional Grammar
//!
//! This module provides a parser for LFG that combines context-free parsing
//! with constraint solving.

use crate::lfg::c_structure::{Category, CNode, CStructure};
use crate::lfg::f_structure::FConstraint;
use crate::lfg::rule::Rule;
use crate::lfg::lexicon::Lexicon;
use crate::lfg::registry::AtomicCategoryRegistry;
use crate::common::Parser as ParserTrait;
use crate::common::error::Error;

/// Configuration options for the LFG parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum depth for the parser
    pub max_depth: usize,
    /// Whether to enable debugging output
    pub debug: bool,
    /// Enforce coherence and completeness
    pub enforce_well_formedness: bool,
    /// Maximum chart size
    pub max_chart_size: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_depth: 20,
            debug: false,
            enforce_well_formedness: true,
            max_chart_size: 1000,
        }
    }
}

/// Parser for Lexical-Functional Grammar
pub struct LFGParser {
    /// Lexicon mapping words to lexical entries
    pub lexicon: Lexicon,
    /// Grammar rules
    pub rules: Vec<Rule>,
    /// Registry of atomic categories
    pub registry: AtomicCategoryRegistry,
    /// Parser configuration
    pub config: ParserConfig,
}

impl LFGParser {
    /// Create a new LFG parser with default configuration
    pub fn new() -> Self {
        Self {
            lexicon: Lexicon::default(),
            rules: Vec::new(),
            registry: AtomicCategoryRegistry::default(),
            config: ParserConfig::default(),
        }
    }
    
    /// Create a new parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        Self {
            lexicon: Lexicon::default(),
            rules: Vec::new(),
            registry: AtomicCategoryRegistry::default(),
            config,
        }
    }
    
    /// Add a new rule to the grammar
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
    
    /// Get all rules with a specific left-hand side
    pub fn get_rules_for_lhs(&self, category: &str) -> Vec<&Rule> {
        self.rules.iter()
            .filter(|rule| rule.lhs.name == category)
            .collect()
    }
    
    /// Get all rules with a specific right-hand side length
    pub fn get_rules_for_rhs_length(&self, length: usize) -> Vec<&Rule> {
        self.rules.iter()
            .filter(|rule| rule.rhs.len() == length)
            .collect()
    }
    
    /// Parse a sentence using a CKY-style algorithm
    pub fn parse_internal(&self, _sentence: &str) -> Option<CStructure> {
        // This is a placeholder implementation
        // In a real implementation, this would contain the parsing logic
        None
    }
    
    /// Add coordination rules
    pub fn add_coordination_rules(&mut self) {
        // NP -> NP CONJ NP  (John and Mary)
        let np = Category::np();
        let conj = Category::conj();
        let mut rule = Rule::new(np.clone(), vec![np.clone(), conj.clone(), np.clone()]);
        
        // Add annotations for coordination
        rule.annotate(0, vec![FConstraint::Containment("↑MEMBERS".to_string(), "↓".to_string())]);
        rule.annotate(2, vec![FConstraint::Containment("↑MEMBERS".to_string(), "↓".to_string())]);
        rule.name = Some("NP -> NP CONJ NP".to_string());
        
        self.add_rule(rule);
        
        // VP -> VP CONJ VP  (runs and jumps)
        let vp = Category::vp();
        let mut rule = Rule::new(vp.clone(), vec![vp.clone(), conj, vp.clone()]);
        
        rule.annotate(0, vec![FConstraint::Containment("↑MEMBERS".to_string(), "↓".to_string())]);
        rule.annotate(2, vec![FConstraint::Containment("↑MEMBERS".to_string(), "↓".to_string())]);
        rule.name = Some("VP -> VP CONJ VP".to_string());
        
        self.add_rule(rule);
    }
    
    /// Add rules for adjuncts
    pub fn add_adjunct_rules(&mut self) {
        // NP -> NP PP  (The cat on the mat)
        let np = Category::np();
        let pp = Category::new("PP");
        let mut rule = Rule::new(np.clone(), vec![np.clone(), pp.clone()]);
        
        rule.annotate(0, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![FConstraint::Containment("↑ADJUNCTS".to_string(), "↓".to_string())]);
        rule.name = Some("NP -> NP PP (Adjunct)".to_string());
        
        self.add_rule(rule);
        
        // VP -> VP PP  (Sleeps on the mat)
        let vp = Category::vp();
        let mut rule = Rule::new(vp.clone(), vec![vp.clone(), pp.clone()]);
        
        rule.annotate(0, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![FConstraint::Containment("↑ADJUNCTS".to_string(), "↓".to_string())]);
        rule.name = Some("VP -> VP PP (Adjunct)".to_string());
        
        self.add_rule(rule);
        
        // VP -> VP ADVP  (Sleeps soundly)
        let advp = Category::new("ADVP");
        let mut rule = Rule::new(vp.clone(), vec![vp.clone(), advp]);
        
        rule.annotate(0, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![FConstraint::Containment("↑ADJUNCTS".to_string(), "↓".to_string())]);
        rule.name = Some("VP -> VP ADVP (Adjunct)".to_string());
        
        self.add_rule(rule);
    }
    
    /// Add rules for control and raising
    pub fn add_control_rules(&mut self) {
        // VP -> V VP  (Subject control: try to sleep)
        let vp = Category::vp();
        let v = Category::new("V");
        let mut rule = Rule::new(vp.clone(), vec![v.clone(), vp.clone()]);
        
        rule.annotate(0, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![
            FConstraint::Equation("↑XCOMP".to_string(), "↓".to_string()),
            FConstraint::Equation("↑SUBJ".to_string(), "↓SUBJ".to_string())
        ]);
        rule.name = Some("VP -> V VP (Control)".to_string());
        
        self.add_rule(rule);
        
        // VP -> V S  (Raising: seem to sleep)
        let s = Category::s();
        let mut rule = Rule::new(vp.clone(), vec![v.clone(), s.clone()]);
        
        rule.annotate(0, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![
            FConstraint::Equation("↑COMP".to_string(), "↓".to_string()),
            FConstraint::Equation("↑SUBJ".to_string(), "↓SUBJ".to_string())
        ]);
        rule.name = Some("VP -> V S (Raising)".to_string());
        
        self.add_rule(rule);
    }
    
    /// Parse with support for empty categories and long-distance dependencies
    pub fn parse_with_empty_categories(&self, sentence: &str) -> Option<CStructure> {
        // Enhanced parsing algorithm that can handle empty categories
        // This would be a more complex implementation of the chart parser
        // that allows for empty productions
        
        // For now, we'll just call the regular parser and convert the result
        self.parse(sentence).map(|node| CStructure { 
            root: node,
            words: sentence.split_whitespace().map(String::from).collect()
        })
    }
}

impl ParserTrait for LFGParser {
    type Cat = Category;
    type Node = CNode;
    type Config = ParserConfig;
    
    fn create_category_with_features(&self, name: &str, features: &[(&str, &str)]) -> Result<Self::Cat, Error> {
        let mut feature_struct = crate::common::FeatureStructure::new();
        for (feature, value) in features {
            feature_struct.add(feature, crate::common::FeatureValue::Atomic(value.to_string()));
        }
        Ok(Category::with_features(name, feature_struct))
    }
    
    fn parse(&self, sentence: &str) -> Option<Self::Node> {
        self.parse_internal(sentence).map(|cs| cs.root)
    }
    
    fn add_to_lexicon(&mut self, word: &str, category: Self::Cat) {
        self.lexicon.add_word(word, category);
    }
    
    fn config(&self) -> &Self::Config {
        &self.config
    }
    
    fn set_config(&mut self, config: Self::Config) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Setup an English lexicon for testing
    pub fn setup_english_lexicon() -> Lexicon {
        let mut lexicon = Lexicon::default();
        
        // Add some basic English words
        lexicon.add_word("the", Category::new("DET"));
        lexicon.add_word("a", Category::new("DET"));
        lexicon.add_word("cat", Category::new("N"));
        lexicon.add_word("dog", Category::new("N"));
        lexicon.add_word("sleeps", Category::new("V"));
        lexicon.add_word("sees", Category::new("V"));
        
        lexicon
    }
    
    /// Setup a comprehensive English lexicon for testing
    pub fn setup_comprehensive_english_lexicon() -> Lexicon {
        let mut lexicon = setup_english_lexicon();
        
        // Add more words
        lexicon.add_word("and", Category::conj());
        lexicon.add_word("or", Category::conj());
        lexicon.add_word("on", Category::new("P"));
        lexicon.add_word("in", Category::new("P"));
        lexicon.add_word("with", Category::new("P"));
        lexicon.add_word("quickly", Category::new("ADV"));
        lexicon.add_word("slowly", Category::new("ADV"));
        
        lexicon
    }
    
    /// Setup a basic grammar with common syntactic rules
    pub fn setup_basic_grammar(parser: &mut LFGParser) {
        // Add basic rules
        
        // S -> NP VP
        let s = Category::new("S");
        let np = Category::new("NP");
        let vp = Category::new("VP");
        let mut rule = Rule::new(s, vec![np.clone(), vp.clone()]);
        rule.annotate(0, vec![FConstraint::Equation("↑SUBJ".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.name = Some("S -> NP VP".to_string());
        parser.add_rule(rule);
        
        // VP -> V
        let v = Category::new("V");
        let mut rule = Rule::new(vp.clone(), vec![v.clone()]);
        rule.annotate(0, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.name = Some("VP -> V".to_string());
        parser.add_rule(rule);
        
        // VP -> V NP
        let mut rule = Rule::new(vp.clone(), vec![v.clone(), np.clone()]);
        rule.annotate(0, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![FConstraint::Equation("↑OBJ".to_string(), "↓".to_string())]);
        rule.name = Some("VP -> V NP".to_string());
        parser.add_rule(rule);
        
        // NP -> Det N
        let det = Category::new("DET");
        let n = Category::new("N");
        let mut rule = Rule::new(np.clone(), vec![det, n.clone()]);
        rule.annotate(0, vec![FConstraint::Equation("↑DET".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.name = Some("NP -> Det N".to_string());
        parser.add_rule(rule);
        
        // NP -> N
        let mut rule = Rule::new(np.clone(), vec![n.clone()]);
        rule.annotate(0, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.name = Some("NP -> N".to_string());
        parser.add_rule(rule);
    }
    
    /// Setup a more comprehensive grammar with extended syntactic rules
    pub fn setup_comprehensive_grammar(parser: &mut LFGParser) {
        // Start with basic grammar
        setup_basic_grammar(parser);
        
        // Add more complex rules
        parser.add_control_rules();
        
        // Add coordination rules
        parser.add_coordination_rules();
        
        // Add rules for adjuncts
        parser.add_adjunct_rules();
        
        // Add rules for long-distance dependencies
        add_long_distance_rules(parser);
    }
    
    /// Add rules for long-distance dependencies
    fn add_long_distance_rules(parser: &mut LFGParser) {
        // S -> NP S  (Topicalization: Mary, John saw)
        let s = Category::new("S");
        let np = Category::new("NP");
        let mut rule = Rule::new(s.clone(), vec![np.clone(), s.clone()]);
        
        // Use functional uncertainty for topicalization
        rule.annotate(0, vec![
            FConstraint::FunctionalUncertainty("↑TOPIC".to_string(), "↓".to_string()),
            FConstraint::FunctionalUncertainty("↑COMP* GF*".to_string(), "↓".to_string())
        ]);
        rule.annotate(1, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.name = Some("S -> NP S (Topicalization)".to_string());
        
        parser.add_rule(rule);
        
        // S -> WH-NP S  (Questions: Who did John see?)
        let mut wh_features = crate::common::FeatureStructure::new();
        wh_features.add("wh", crate::common::FeatureValue::Atomic("yes".to_string()));
        let wh_np = Category::with_features("NP", wh_features);
        let mut rule = Rule::new(s.clone(), vec![wh_np, s.clone()]);
        
        rule.annotate(0, vec![
            FConstraint::FunctionalUncertainty("↑FOCUS".to_string(), "↓".to_string()),
            FConstraint::FunctionalUncertainty("↑COMP* GF*".to_string(), "↓".to_string())
        ]);
        rule.annotate(1, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.name = Some("S -> WH-NP S (Question)".to_string());
        
        parser.add_rule(rule);
    }
    
    /// Setup an English grammar for testing
    pub fn setup_english_grammar(parser: &mut LFGParser) {
        // Set up the lexicon
        parser.lexicon = setup_english_lexicon();
        
        // Add basic rules
        setup_basic_grammar(parser);
    }
    
    /// Setup a comprehensive English grammar for testing
    pub fn setup_comprehensive_english_grammar(parser: &mut LFGParser) {
        // Set up the lexicon
        parser.lexicon = setup_comprehensive_english_lexicon();
        
        // Add comprehensive rules
        setup_comprehensive_grammar(parser);
    }
    
    // Helper function to set up a basic parser for testing
    fn setup_test_parser() -> LFGParser {
        let mut parser = LFGParser::new();
        setup_english_grammar(&mut parser);
        parser
    }
    
    #[test]
    fn test_parser_creation() {
        let parser = LFGParser::new();
        
        assert!(parser.lexicon.is_empty());
        assert!(parser.rules.is_empty());
        assert!(!parser.registry.is_empty()); // Default registry has standard categories
    }
    
    #[test]
    fn test_setup_english_grammar() {
        let mut parser = LFGParser::new();
        setup_english_grammar(&mut parser);
        
        // Check that the lexicon is populated
        assert!(!parser.lexicon.is_empty());
        assert!(parser.lexicon.contains("cat"));
        
        // Check that rules are added
        assert!(!parser.rules.is_empty());
        
        // Check for specific rules
        let s_rules = parser.get_rules_for_lhs("S");
        assert!(!s_rules.is_empty());
        
        let vp_rules = parser.get_rules_for_lhs("VP");
        assert!(!vp_rules.is_empty());
        
        let np_rules = parser.get_rules_for_lhs("NP");
        assert!(!np_rules.is_empty());
    }
    
    #[test]
    fn test_basic_parsing() {
        let parser = setup_test_parser();
        
        // Test a simple sentence
        let result = parser.parse("the cat sleeps");
        assert!(result.is_some());
        
        let parse_tree = result.unwrap();
        assert_eq!(parse_tree.category.name, "S");
        assert_eq!(parse_tree.children.len(), 2);
        assert_eq!(parse_tree.children[0].category.name, "NP");
        assert_eq!(parse_tree.children[1].category.name, "VP");
    }
    
    #[test]
    fn test_parse_with_constraints() {
        let mut parser = setup_test_parser();
        
        // Enable constraint checking
        parser.config.enforce_well_formedness = true;
        
        // Test a grammatical sentence
        let result = parser.parse("the cat sleeps");
        assert!(result.is_some());
        
        let parse_tree = result.unwrap();
        
        // Check that F-structures were created
        assert!(parse_tree.f_structure.is_some());
        assert!(parse_tree.children[0].f_structure.is_some()); // NP
        assert!(parse_tree.children[1].f_structure.is_some()); // VP
        
        // Test an ungrammatical sentence (number agreement violation)
        let result = parser.parse("the cat sleep");
        assert!(result.is_none());
    }
    
    #[test]
    fn test_advanced_parsing() {
        let parser = setup_test_parser();
        
        // Test a more complex sentence
        let result = parser.parse("the cat sees the dog");
        assert!(result.is_some());
        
        let parse_tree = result.unwrap();
        assert_eq!(parse_tree.category.name, "S");
        
        // Check the VP child has a V and NP (object)
        let vp = &parse_tree.children[1];
        assert_eq!(vp.category.name, "VP");
        assert_eq!(vp.children.len(), 2);
        assert_eq!(vp.children[0].category.name, "V");
        assert_eq!(vp.children[1].category.name, "NP");
    }
    
    #[test]
    fn test_unknown_word() {
        let parser = setup_test_parser();
        
        // Test with an unknown word
        let result = parser.parse("the cat zzzyyyxxx");
        assert!(result.is_none());
    }
    
    #[test]
    fn test_config_options() {
        let mut config = ParserConfig::default();
        config.debug = true;
        config.enforce_well_formedness = false;
        
        let parser = LFGParser::with_config(config);
        
        assert!(parser.config.debug);
        assert!(!parser.config.enforce_well_formedness);
    }
}