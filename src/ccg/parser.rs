//! CCG parser implementation

use std::any::Any;
use crate::ccg::category::CCGCategory;
use crate::ccg::node::CCGNode;
use crate::ccg::rules::*;
use crate::common::{Lexicon, AtomicTypeRegistry, FeatureRegistry, FeatureStructure, FeatureValue, Parser};

/// Configuration options for the CCG parser
#[derive(Debug, Clone)]
pub struct CCGParserConfig {
    /// Maximum order of composition allowed
    pub max_composition_order: usize,
    /// Enable type-raising
    pub enable_type_raising: bool,
    /// Target categories for type-raising (S, NP, etc.)
    pub type_raising_targets: Vec<CCGCategory>,
    /// Whether to enforce feature unification
    pub enforce_feature_unification: bool,
    /// Whether to use morphosyntactic features
    pub use_morphosyntax: bool,
}

impl Default for CCGParserConfig {
    fn default() -> Self {
        Self {
            max_composition_order: 2,
            enable_type_raising: true,
            type_raising_targets: vec![CCGCategory::s()],
            enforce_feature_unification: false,
            use_morphosyntax: false,
        }
    }
}

/// A trait object wrapper that can be downcasted
trait RuleObj: CCGRule + Any {
    fn as_any(&mut self) -> &mut dyn Any;
}

impl<T: CCGRule + Any> RuleObj for T {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// The CCG Parser with morphosyntactic features
pub struct CCGParser {
    pub lexicon: Lexicon<CCGCategory>,
    pub atomic_types: AtomicTypeRegistry,
    pub feature_registry: FeatureRegistry,
    pub config: CCGParserConfig,
    rules: Vec<Box<dyn RuleObj>>,
}

impl CCGParser {
    /// Create a new CCG parser with default configuration
    pub fn new() -> Self {
        let config = CCGParserConfig::default();
        
        // Initialize with standard rules
        let rules: Vec<Box<dyn RuleObj>> = vec![
            Box::new(ForwardApplication),
            Box::new(BackwardApplication),
            Box::new(ForwardComposition),
            Box::new(BackwardComposition),
            Box::new(ForwardTypeRaising { 
                targets: config.type_raising_targets.clone() 
            }),
            Box::new(BackwardTypeRaising { 
                targets: config.type_raising_targets.clone() 
            }),
        ];
        
        CCGParser {
            lexicon: Lexicon::new(),
            atomic_types: AtomicTypeRegistry::new(),
            feature_registry: FeatureRegistry::new(),
            config,
            rules,
        }
    }
    
    /// Create a new parser with custom configuration
    pub fn with_config(config: CCGParserConfig) -> Self {
        let mut parser = Self::new();
        parser.config = config;
        
        // Update type-raising rules with new targets
        for rule in &mut parser.rules {
            if let Some(tr_rule) = rule.as_any().downcast_mut::<ForwardTypeRaising>() {
                tr_rule.targets = parser.config.type_raising_targets.clone();
            } else if let Some(tr_rule) = rule.as_any().downcast_mut::<BackwardTypeRaising>() {
                tr_rule.targets = parser.config.type_raising_targets.clone();
            }
        }
        
        parser
    }
    
    /// Register a new atomic type
    pub fn register_atomic_type(&mut self, type_name: &str) {
        self.atomic_types.register(type_name);
    }
    
    /// Register a new feature dimension
    pub fn register_feature_dimension(&mut self, feature: &str, values: &[&str]) {
        self.feature_registry.register_feature(feature, values);
    }
    
    /// Create a category using a registered atomic type
    pub fn create_atomic_category(&self, type_name: &str) -> Option<CCGCategory> {
        if self.atomic_types.is_registered(type_name) {
            Some(CCGCategory::atomic(type_name))
        } else {
            eprintln!("Warning: Unregistered atomic type '{}'. Register it first with register_atomic_type.", type_name);
            None
        }
    }
    
    /// Create a category with features
    pub fn create_category_with_features(&self, type_name: &str, features: &[(&str, &str)]) -> Option<CCGCategory> {
        if !self.atomic_types.is_registered(type_name) {
            eprintln!("Warning: Unregistered atomic type '{}'. Register it first with register_atomic_type.", type_name);
            return None;
        }
        
        let mut feature_struct = FeatureStructure::new();
        
        for (feature, value) in features {
            if !self.feature_registry.is_feature_registered(feature) {
                eprintln!("Warning: Unregistered feature '{}'. Register it first with register_feature_dimension.", feature);
                return None;
            }
            
            if !self.feature_registry.is_value_valid(feature, value) {
                eprintln!("Warning: Invalid value '{}' for feature '{}'. Check registered values.", value, feature);
                return None;
            }
            
            feature_struct.add(feature, FeatureValue::Atomic(value.to_string()));
        }
        
        Some(CCGCategory::atomic_with_features(type_name, feature_struct))
    }
    
    /// Validate that all atomic types in a category are registered
    fn validate_category(&self, category: &CCGCategory) -> bool {
        match category {
            CCGCategory::Atomic(name, features) => {
                if !self.atomic_types.is_registered(name) {
                    eprintln!("Unregistered atomic type: {}", name);
                    return false;
                }
                
                // Check if all features are valid
                for (feature_name, _) in &features.features {
                    if !self.feature_registry.is_feature_registered(feature_name) {
                        eprintln!("Unregistered feature: {}", feature_name);
                        return false;
                    }
                }
                
                true
            },
            CCGCategory::Forward(left, right) => {
                self.validate_category(left) && self.validate_category(right)
            },
            CCGCategory::Backward(left, right) => {
                self.validate_category(left) && self.validate_category(right)
            },
        }
    }
    
    /// Parse a sentence using the CKY algorithm with CCG combinatory rules
    fn parse_internal(&self, sentence: &str) -> Option<CCGNode> {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        let n = words.len();
        
        // Initialize the chart for CKY parsing
        let mut chart = vec![vec![vec![]; n + 1]; n + 1];
        
        // Fill in the lexical entries (diagonal)
        for i in 0..n {
            let word = words[i];
            let categories = self.lexicon.get_categories(word);
            
            if categories.is_empty() {
                eprintln!("Unknown word: {}", word);
                return None;
            }
            
            for category in categories {
                chart[i][i + 1].push(CCGNode::leaf(word, category));
            }
        }
        
        // Fill in the chart using CCG combinatory rules
        for span in 2..=n {
            for start in 0..=(n - span) {
                let end = start + span;
                
                for split in (start + 1)..end {
                    // For each pair of adjacent cells in the chart
                    let mut new_nodes = Vec::new();
                    
                    for left in &chart[start][split] {
                        for right in &chart[split][end] {
                            // Apply all available rules
                            for rule in &self.rules {
                                if let Some(node) = rule.apply(
                                    left, 
                                    right, 
                                    self.config.use_morphosyntax && self.config.enforce_feature_unification
                                ) {
                                    new_nodes.push(node);
                                }
                            }
                            
                            // Try generalized composition if needed
                            if self.config.max_composition_order > 1 {
                                if let Some(node) = self.compose_forward_generalized(
                                    left, 
                                    right, 
                                    self.config.max_composition_order
                                ) {
                                    new_nodes.push(node);
                                }
                                
                                if let Some(node) = self.compose_backward_generalized(
                                    left,
                                    right,
                                    self.config.max_composition_order
                                ) {
                                    new_nodes.push(node);
                                }
                            }
                        }
                    }
                    
                    chart[start][end].extend(new_nodes);
                }
            }
        }
        
        // Find a complete parse (category S spanning the whole sentence)
        for node in &chart[0][n] {
            if let CCGCategory::Atomic(s, _) = &node.category {
                if s == "S" {
                    return Some(node.clone());
                }
            }
        }
        
        // No complete parse found
        eprintln!("No complete parse found for: {}", sentence);
        if !chart[0][n].is_empty() {
            eprintln!("Partial parses:");
            for (i, node) in chart[0][n].iter().enumerate() {
                eprintln!("Parse {}: {}", i + 1, node.category);
            }
        }
        
        None
    }
    
    /// Forward generalized composition (order n): X/Y Y... => X...
    /// Only the first slash needs to match (Y argument type)
    fn compose_forward_generalized(&self, left: &CCGNode, right: &CCGNode, max_order: usize) -> Option<CCGNode> {
        // Basic check for forward slash in the left category
        if let CCGCategory::Forward(x, y) = &left.category {
            let _matches = if self.config.use_morphosyntax && self.config.enforce_feature_unification {
                // Try to unify the argument category with the right-hand category's main type
                match &right.category {
                    CCGCategory::Forward(right_result, _) => y.unify(right_result).is_some(),
                    CCGCategory::Backward(right_result, _) => y.unify(right_result).is_some(),
                    _ => y.unify(&right.category).is_some(),
                }
            } else {
                // Simple equality check
                match &right.category {
                    CCGCategory::Forward(right_result, _) => **y == **right_result,
                    CCGCategory::Backward(right_result, _) => **y == **right_result,
                    _ => **y == right.category,
                }
            };
            
            // Only try higher-order composition (we already have first-order via rules)
            if max_order > 1 {
                // Extract the functor chain from the right category
                if let Some((right_base, right_args)) = extract_category_chain(&right.category, 0, max_order) {
                    // Check if y matches the base result of the right category
                    let base_matches = if self.config.use_morphosyntax && self.config.enforce_feature_unification {
                        y.unify(&right_base).is_some()
                    } else {
                        **y == right_base
                    };
                    
                    if base_matches && right_args.len() > 1 {
                        // Construct the result category by combining X with all arguments from right
                        let mut result = (**x).clone();
                        
                        // Build the category by applying arguments in reverse order 
                        // (deepest arguments first)
                        for (is_forward, arg) in right_args.iter().rev() {
                            if *is_forward {
                                result = CCGCategory::forward(result, arg.clone());
                            } else {
                                result = CCGCategory::backward(result, arg.clone());
                            }
                        }
                        
                        return Some(CCGNode::internal(
                            result,
                            vec![left.clone(), right.clone()],
                            &format!(">B{}", right_args.len()), // Order is number of args
                        ));
                    }
                }
            }
        }
        
        None
    }
    
    /// Backward generalized composition (order n): Y... X\Y => X...
    /// Only the first slash needs to match (Y argument type)
    fn compose_backward_generalized(&self, left: &CCGNode, right: &CCGNode, max_order: usize) -> Option<CCGNode> {
        // Basic check for backward slash in the right category
        if let CCGCategory::Backward(x, y) = &right.category {
            let _matches = if self.config.use_morphosyntax && self.config.enforce_feature_unification {
                // Try to unify the argument category with the left-hand category's main type
                match &left.category {
                    CCGCategory::Forward(left_result, _) => y.unify(left_result).is_some(),
                    CCGCategory::Backward(left_result, _) => y.unify(left_result).is_some(),
                    _ => y.unify(&left.category).is_some(),
                }
            } else {
                // Simple equality check
                match &left.category {
                    CCGCategory::Forward(left_result, _) => **y == **left_result,
                    CCGCategory::Backward(left_result, _) => **y == **left_result,
                    _ => **y == left.category,
                }
            };
            
            // Only try higher-order composition (we already have first-order via rules)
            if max_order > 1 {
                // Extract the functor chain from the left category
                if let Some((left_base, left_args)) = extract_category_chain(&left.category, 0, max_order) {
                    // Check if y matches the base result of the left category
                    let base_matches = if self.config.use_morphosyntax && self.config.enforce_feature_unification {
                        y.unify(&left_base).is_some()
                    } else {
                        **y == left_base
                    };
                    
                    if base_matches && left_args.len() > 1 {
                        // Construct the result category by combining X with all arguments from left
                        let mut result = (**x).clone();
                        
                        // Build the category by applying arguments in reverse order
                        // (deepest arguments first)
                        for (is_forward, arg) in left_args.iter().rev() {
                            if *is_forward {
                                result = CCGCategory::forward(result, arg.clone());
                            } else {
                                result = CCGCategory::backward(result, arg.clone());
                            }
                        }
                        
                        return Some(CCGNode::internal(
                            result,
                            vec![left.clone(), right.clone()],
                            &format!("<B{}", left_args.len()), // Order is number of args
                        ));
                    }
                }
            }
        }
        
        None
    }
}

impl Parser for CCGParser {
    type Cat = CCGCategory;
    type Node = CCGNode;
    type Config = CCGParserConfig;
    
    fn create_category_with_features(&self, type_name: &str, features: &[(&str, &str)]) -> Result<Self::Cat, crate::common::error::Error> {
        if !self.atomic_types.is_registered(type_name) {
            return Err(crate::common::error::Error::UnregisteredType(type_name.to_string()));
        }
        
        let mut feature_struct = FeatureStructure::new();
        
        for (feature, value) in features {
            if !self.feature_registry.is_feature_registered(feature) {
                return Err(crate::common::error::Error::UnregisteredFeature(feature.to_string()));
            }
            
            if !self.feature_registry.is_value_valid(feature, value) {
                return Err(crate::common::error::Error::InvalidFeatureValue {
                    feature: feature.to_string(), 
                    value: value.to_string()
                });
            }
            
            feature_struct.add(feature, FeatureValue::Atomic(value.to_string()));
        }
        
        Ok(CCGCategory::atomic_with_features(type_name, feature_struct))
    }
    
    /// Parse a sentence and return a parse tree if successful
    fn parse(&self, sentence: &str) -> Option<Self::Node> {
        self.parse_internal(sentence)
    }
    
    /// Add a word with a category to the lexicon
    fn add_to_lexicon(&mut self, word: &str, category: Self::Cat) {
        // Validate that all atomic types used in the category are registered
        if self.validate_category(&category) {
            self.lexicon.add(word, category);
        } else {
            eprintln!("Warning: Category for '{}' contains unregistered atomic types.", word);
        }
    }
    
    /// Get the configuration of this parser
    fn config(&self) -> &Self::Config {
        &self.config
    }
    
    /// Set the configuration of this parser
    fn set_config(&mut self, config: Self::Config) {
        self.config = config;
        
        // Update type-raising rules with new targets
        for rule in &mut self.rules {
            if let Some(tr_rule) = rule.as_any().downcast_mut::<ForwardTypeRaising>() {
                tr_rule.targets = self.config.type_raising_targets.clone();
            } else if let Some(tr_rule) = rule.as_any().downcast_mut::<BackwardTypeRaising>() {
                tr_rule.targets = self.config.type_raising_targets.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to set up a basic English parser for testing
    fn setup_test_parser() -> CCGParser {
        let mut parser = CCGParser::new();
        
        // Register basic atomic types
        parser.register_atomic_type("S");
        parser.register_atomic_type("NP");
        parser.register_atomic_type("N");
        
        // Add basic lexical entries
        let s = parser.create_atomic_category("S").unwrap();
        let np = parser.create_atomic_category("NP").unwrap();
        let n = parser.create_atomic_category("N").unwrap();
        
        // Determiners
        parser.add_to_lexicon("the", CCGCategory::forward(np.clone(), n.clone()));
        parser.add_to_lexicon("a", CCGCategory::forward(np.clone(), n.clone()));
        
        // Nouns
        parser.add_to_lexicon("cat", n.clone());
        parser.add_to_lexicon("dog", n.clone());
        
        // Intransitive verbs
        parser.add_to_lexicon("sleeps", CCGCategory::backward(s.clone(), np.clone()));
        parser.add_to_lexicon("runs", CCGCategory::backward(s.clone(), np.clone()));
        
        // Transitive verbs
        let tv_type = CCGCategory::backward(
            CCGCategory::backward(s.clone(), np.clone()),
            np.clone()
        );
        parser.add_to_lexicon("chases", tv_type.clone());
        parser.add_to_lexicon("sees", tv_type.clone());
        
        parser
    }
    
    #[test]
    fn test_basic_parsing() {
        let parser = setup_test_parser();
        
        // Test parsing basic sentences
        let result = parser.parse("the cat sleeps");
        assert!(result.is_some());
        
        let result = parser.parse("the dog chases the cat");
        assert!(result.is_some());
    }
    
    #[test]
    fn test_failed_parse() {
        let parser = setup_test_parser();
        
        // Test parsing ungrammatical sentences
        let result = parser.parse("the sleeps cat");
        assert!(result.is_none());
        
        let result = parser.parse("cat the sleeps");
        assert!(result.is_none());
    }
    
    #[test]
    fn test_morphosyntax_parsing() {
        let mut parser = setup_test_parser();
        
        // Enable morphosyntactic features
        let mut config = CCGParserConfig::default();
        config.use_morphosyntax = true;
        config.enforce_feature_unification = true;
        parser.config = config;
        
        // Register features
        parser.register_feature_dimension("num", &["sg", "pl"]);
        parser.register_feature_dimension("per", &["1", "2", "3"]);
        
        // Add feature-enhanced entries
        let sg_n = parser.create_category_with_features("N", &[("num", "sg")]).unwrap();
        let pl_n = parser.create_category_with_features("N", &[("num", "pl")]).unwrap();
        
        let np_sg = parser.create_category_with_features("NP", &[("num", "sg")]).unwrap();
        let np_pl = parser.create_category_with_features("NP", &[("num", "pl")]).unwrap();
        
        // Determiners with agreement
        parser.add_to_lexicon("a", CCGCategory::forward(np_sg.clone(), sg_n.clone()));
        parser.add_to_lexicon("some", CCGCategory::forward(np_pl.clone(), pl_n.clone()));
        
        // Nouns with number
        parser.add_to_lexicon("cat", sg_n.clone());
        parser.add_to_lexicon("cats", pl_n.clone());
        
        // Verbs with agreement
        let s = parser.create_atomic_category("S").unwrap();
        parser.add_to_lexicon("sleeps", CCGCategory::backward(s.clone(), np_sg));
        parser.add_to_lexicon("sleep", CCGCategory::backward(s.clone(), np_pl));
        
        // Test grammatical sentences
        let result = parser.parse("a cat sleeps");
        assert!(result.is_some());
        
        let result = parser.parse("some cats sleep");
        assert!(result.is_some());
        
        // Test ungrammatical sentences
        let result = parser.parse("a cats sleeps");
        assert!(result.is_none());
        
        let result = parser.parse("some cat sleep");
        assert!(result.is_none());
    }
    
    #[test]
    fn test_composition_rules() {
        let mut parser = setup_test_parser();
        
        // Add entries to test composition
        let s = parser.create_atomic_category("S").unwrap();
        let np = parser.create_atomic_category("NP").unwrap();
        let _n = parser.create_atomic_category("N").unwrap();
        
        // Add an auxiliary verb
        let aux_type = CCGCategory::backward(
            CCGCategory::forward(s.clone(), 
                CCGCategory::backward(s.clone(), np.clone())),
            np.clone()
        );
        parser.add_to_lexicon("will", aux_type);
        
        // Test sentence requiring composition
        let result = parser.parse("the cat will sleep");
        assert!(result.is_some());
    }
}