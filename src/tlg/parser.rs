//! Parser for Type-Logical Grammar
//!
//! This module provides the main parser for Type-Logical Grammar, using
//! either natural deduction or proof nets to derive semantic representations.

use std::collections::VecDeque;
use crate::common::{FeatureRegistry, FeatureValue, FeatureStructure};
use crate::tlg::logical_type::LogicalType;
use crate::tlg::modality::Modality;
use crate::tlg::proof::{ProofNode, ProofSearchState};
use crate::tlg::proof_net::ProofNet;
use crate::tlg::registry::AtomicTypeRegistry;
use crate::tlg::lexicon::Lexicon;
use crate::common::Parser as ParserTrait;

/// Configuration options for the parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum depth for the search
    pub max_depth: usize,
    /// Whether to use product types
    pub use_product: bool,
    /// Whether to use modal operators
    pub use_modalities: bool,
    /// Whether to use first-order quantifiers
    pub use_quantifiers: bool,
    /// Whether to use strict linear logic (no resource duplication)
    pub strict_linear: bool,
    /// Logic variant to use (e.g., "NL", "L", "NL(3)", etc.)
    pub logic_variant: String,
    /// Whether to use proof nets for parsing (more efficient)
    pub use_proof_nets: bool,
    /// Enable Displacement Calculus
    pub use_displacement: bool,
    /// Enable linguistic features
    pub use_features: bool,
    /// Available modalities for multi-modal system
    pub modalities: Vec<Modality>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_depth: 20,
            use_product: true,
            use_modalities: false,
            use_quantifiers: false,
            strict_linear: true,
            logic_variant: "NL".to_string(), // Non-associative Lambek calculus by default
            use_proof_nets: false,
            use_displacement: false,
            use_features: true,
            modalities: vec![],
        }
    }
}

/// Type-Logical Grammar Parser
pub struct TLGParser {
    /// The lexicon mapping words to logical types
    pub lexicon: Lexicon,
    /// Registry of atomic types
    pub atomic_types: AtomicTypeRegistry,
    /// Configuration for the parser
    pub config: ParserConfig,
    /// Registry for linguistic features
    pub feature_registry: FeatureRegistry,
}

impl TLGParser {
    /// Create a new TLG parser with default configuration
    pub fn new() -> Self {
        let mut parser = Self {
            lexicon: Lexicon::new(),
            atomic_types: AtomicTypeRegistry::default(),
            config: ParserConfig::default(),
            feature_registry: FeatureRegistry::new(),
        };
        
        // Populate the lexicon with some basic entries
        parser.populate_basic_lexicon();
        
        parser
    }
    
    /// Create a new parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        let mut parser = Self::new();
        parser.config = config;
        parser
    }
    
    /// Register a new atomic type
    pub fn register_atomic_type(&mut self, type_name: &str) {
        self.atomic_types.register(type_name);
    }
    
    /// Register a linguistic feature
    pub fn register_feature(&mut self, name: &str, values: &[&str]) {
        self.feature_registry.register_feature(name, values);
    }
    
    /// Register a new modality
    pub fn register_modality(&mut self, index: usize, properties: Vec<crate::tlg::logical_type::StructuralProperty>) {
        let modality = Modality::with_properties(index, properties);
        self.config.modalities.push(modality);
    }
    
    /// Create a basic lexicon for English
    fn populate_basic_lexicon(&mut self) {
        // Function to create common type combinations
        let np = LogicalType::np();
        let s = LogicalType::s();
        let n = LogicalType::n();
        
        // Determiners: (np ← n)
        let det_type = LogicalType::left_impl(np.clone(), n.clone());
        self.lexicon.add("the", det_type.clone());
        self.lexicon.add("a", det_type.clone());
        
        // Nouns: n
        self.lexicon.add("cat", n.clone());
        self.lexicon.add("dog", n.clone());
        self.lexicon.add("man", n.clone());
        self.lexicon.add("woman", n.clone());
        
        // Add feature-enriched entries if features are enabled
        if self.config.use_features {
            // Create feature structures
            let mut sg_feat = FeatureStructure::new();
            sg_feat.add("num", FeatureValue::Atomic("sg".to_string()));
            
            let mut pl_feat = FeatureStructure::new();
            pl_feat.add("num", FeatureValue::Atomic("pl".to_string()));
            
            // Singular nouns
            let n_sg = LogicalType::atomic_with_features("n", &sg_feat);
            self.lexicon.add("cat", n_sg.clone());
            self.lexicon.add("dog", n_sg.clone());
            
            // Plural nouns
            let n_pl = LogicalType::atomic_with_features("n", &pl_feat);
            self.lexicon.add("cats", n_pl.clone());
            self.lexicon.add("dogs", n_pl.clone());
            
            // Agreement-sensitive determiners
            let np_sg = LogicalType::atomic_with_features("np", &sg_feat);
            let det_sg = LogicalType::left_impl(np_sg.clone(), n_sg.clone());
            self.lexicon.add("a", det_sg);
            
            let np_pl = LogicalType::atomic_with_features("np", &pl_feat);
            let det_pl = LogicalType::left_impl(np_pl.clone(), n_pl.clone());
            self.lexicon.add("some", det_pl);
            
            // Create person features
            let mut third_sg_feat = sg_feat.clone();
            third_sg_feat.add("per", FeatureValue::Atomic("3".to_string()));
            
            let mut third_pl_feat = pl_feat.clone();
            third_pl_feat.add("per", FeatureValue::Atomic("3".to_string()));
            
            // Verbs with agreement
            let s_plain = LogicalType::s();
            let np_3sg = LogicalType::atomic_with_features("np", &third_sg_feat);
            let np_3pl = LogicalType::atomic_with_features("np", &third_pl_feat);
            
            // Intransitive verbs
            let iv_3sg = LogicalType::left_impl(s_plain.clone(), np_3sg.clone());
            let iv_3pl = LogicalType::left_impl(s_plain.clone(), np_3pl.clone());
            
            self.lexicon.add("sleeps", iv_3sg.clone());
            self.lexicon.add("runs", iv_3sg.clone());
            self.lexicon.add("sleep", iv_3pl.clone());
            self.lexicon.add("run", iv_3pl.clone());
            
            // Transitive verbs
            let tv_3sg = LogicalType::left_impl(iv_3sg.clone(), np.clone());
            let tv_3pl = LogicalType::left_impl(iv_3pl.clone(), np.clone());
            
            self.lexicon.add("chases", tv_3sg.clone());
            self.lexicon.add("sees", tv_3sg.clone());
            self.lexicon.add("chase", tv_3pl.clone());
            self.lexicon.add("see", tv_3pl.clone());
        }
        
        // Basic entries without features
        
        // Intransitive verbs: (s ← np)
        let iv_type = LogicalType::left_impl(s.clone(), np.clone());
        self.lexicon.add("sleeps", iv_type.clone());
        self.lexicon.add("runs", iv_type.clone());
        
        // Transitive verbs: ((s ← np) ← np)
        let tv_type = LogicalType::left_impl(iv_type.clone(), np.clone());
        self.lexicon.add("sees", tv_type.clone());
        self.lexicon.add("chases", tv_type.clone());
        
        // Adjectives: (n ← n)
        let adj_type = LogicalType::left_impl(n.clone(), n.clone());
        self.lexicon.add("big", adj_type.clone());
        self.lexicon.add("small", adj_type.clone());
        
        // Prepositions: ((n ← n) ← np)
        let prep_type = LogicalType::left_impl(adj_type.clone(), np.clone());
        self.lexicon.add("with", prep_type.clone());
        self.lexicon.add("in", prep_type.clone());
        
        // Add modality-based entries if enabled
        if self.config.use_modalities && !self.config.modalities.is_empty() {
            // Get the first modality for demonstration
            let modality = self.config.modalities[0].clone();
            
            // Create modality-sensitive types
            let iv_modal = LogicalType::left_impl_with_modality(s.clone(), np.clone(), modality.clone());
            self.lexicon.add("walks", iv_modal.clone());
            
            // Intensional transitive verbs: ((s ← np) ← ◇np)
            let np_diamond = LogicalType::diamond(np.clone());
            let int_tv_type = LogicalType::left_impl(iv_type.clone(), np_diamond);
            self.lexicon.add("seeks", int_tv_type.clone());
            self.lexicon.add("needs", int_tv_type.clone());
        }
        
        // Add displacement calculus entries if enabled
        if self.config.use_displacement {
            // Examples of discontinuous types for wh-words, etc.
            let wh_type = LogicalType::up_arrow(s.clone(), np.clone(), 1);
            self.lexicon.add("who", wh_type.clone());
            self.lexicon.add("what", wh_type.clone());
            
            // Extraction verbs
            let extract_type = LogicalType::down_arrow(
                LogicalType::left_impl(s.clone(), np.clone()),
                np.clone(),
                1
            );
            self.lexicon.add("thinks", extract_type.clone());
        }
        
        // Add quantifiers if enabled
        if self.config.use_quantifiers {
            // Universal quantifier: (s → (s ← ∀x.(x → np)))
            let univ_quant = LogicalType::right_impl(
                s.clone(),
                LogicalType::left_impl(
                    s.clone(),
                    LogicalType::Universal(
                        "x".to_string(),
                        Box::new(LogicalType::right_impl(
                            LogicalType::Atomic("x".to_string(), FeatureStructure::new()),
                            np.clone()
                        ))
                    )
                )
            );
            self.lexicon.add("every", univ_quant.clone());
            
            // Existential quantifier: (s → (s ← ∃x.(x → np)))
            let exist_quant = LogicalType::right_impl(
                s.clone(),
                LogicalType::left_impl(
                    s.clone(),
                    LogicalType::Existential(
                        "x".to_string(),
                        Box::new(LogicalType::right_impl(
                            LogicalType::Atomic("x".to_string(), FeatureStructure::new()),
                            np.clone()
                        ))
                    )
                )
            );
            self.lexicon.add("some", exist_quant.clone());
        }
    }
    
    /// Add a word with its logical type to the lexicon
    pub fn add_to_lexicon(&mut self, word: &str, logical_type: LogicalType) {
        // Validate the logical type first
        if self.validate_type(&logical_type) {
            self.lexicon.add(word, logical_type);
        } else {
            eprintln!("Warning: Invalid logical type for '{}'.", word);
        }
    }
    
    /// Add a word with its logical type and phonological form to the lexicon
    pub fn add_to_lexicon_with_phonology(&mut self, word: &str, logical_type: LogicalType, phon: &str) {
        // Validate the logical type first
        if self.validate_type(&logical_type) {
            self.lexicon.add_with_phonology(word, logical_type, phon);
        } else {
            eprintln!("Warning: Invalid logical type for '{}'.", word);
        }
    }
    
    /// Validate a logical type (check that all atomic types are registered)
    fn validate_type(&self, logical_type: &LogicalType) -> bool {
        match logical_type {
            LogicalType::Atomic(name, features) => {
                if !self.atomic_types.is_registered(name) {
                    eprintln!("Unregistered atomic type: {}", name);
                    return false;
                }
                
                // Validate features if using features
                if self.config.use_features && !features.features.is_empty() {
                    for (fname, fvalue) in &features.features {
                        if !self.feature_registry.is_feature_registered(fname) {
                            eprintln!("Unregistered feature: {}", fname);
                            return false;
                        }
                        
                        // Validate atomic feature values
                        if let FeatureValue::Atomic(val) = fvalue {
                            if !self.feature_registry.is_value_valid(fname, val) {
                                eprintln!("Invalid value '{}' for feature '{}'", val, fname);
                                return false;
                            }
                        }
                    }
                }
                
                true
            },
            LogicalType::RightImplication(a, b, modality) |
            LogicalType::LeftImplication(a, b, modality) |
            LogicalType::Product(a, b, modality) => {
                // Validate modality if present
                if let Some(m) = modality {
                    if !self.config.use_modalities {
                        eprintln!("Modalities are not enabled in the current configuration");
                        return false;
                    }
                    
                    if !self.config.modalities.iter().any(|mod_i| mod_i.index == m.index) {
                        eprintln!("Unregistered modality index: {}", m.index);
                        return false;
                    }
                }
                
                self.validate_type(a) && self.validate_type(b)
            },
            LogicalType::Diamond(a, modality) | LogicalType::Box(a, modality) => {
                if !self.config.use_modalities {
                    eprintln!("Modal operators are not enabled in the current configuration");
                    return false;
                }
                
                // Validate modality if present
                if let Some(m) = modality {
                    if !self.config.modalities.iter().any(|mod_i| mod_i.index == m.index) {
                        eprintln!("Unregistered modality index: {}", m.index);
                        return false;
                    }
                }
                
                self.validate_type(a)
            },
            LogicalType::Universal(_, a) | LogicalType::Existential(_, a) => {
                if !self.config.use_quantifiers {
                    eprintln!("Quantifiers are not enabled in the current configuration");
                    return false;
                }
                self.validate_type(a)
            },
            LogicalType::UpArrow(a, b, _) | LogicalType::DownArrow(a, b, _) => {
                if !self.config.use_displacement {
                    eprintln!("Displacement Calculus is not enabled in the current configuration");
                    return false;
                }
                self.validate_type(a) && self.validate_type(b)
            },
        }
    }
    
    /// Parse a sentence using natural deduction for Type-Logical Grammar
    pub fn parse_with_natural_deduction(&self, sentence: &str) -> Option<ProofNode> {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        
        // Create axioms from lexical entries
        let mut axioms = Vec::new();
        for word in &words {
            let items = self.lexicon.get_items(word);
            
            if items.is_empty() {
                eprintln!("Unknown word: {}", word);
                return None;
            }
            
            for item in items {
                axioms.push(ProofNode::axiom(word, item.logical_type));
            }
        }
        
        // Try to derive a complete proof
        self.prove_sentence(&axioms, &LogicalType::s())
    }
    
    /// Parse using proof nets for efficiency
    pub fn parse_with_proof_nets(&self, sentence: &str) -> Option<ProofNode> {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        
        // For each word, create all possible proof nets from its lexical types
        let mut word_nets = Vec::new();
        
        for word in &words {
            let items = self.lexicon.get_items(word);
            
            if items.is_empty() {
                eprintln!("Unknown word: {}", word);
                return None;
            }
            
            let mut nets = Vec::new();
            for item in items {
                // Create a proof net for this lexical type
                let net = ProofNet::from_type(&item.logical_type, true);
                nets.push((word, net));
            }
            
            word_nets.push(nets);
        }
        
        // Now try to link these nets together
        // This is a simplified approach; a real implementation would be more complex
        
        // For example, try to link the first word to the goal type (sentence)
        let goal_type = LogicalType::s();
        let _goal_net = ProofNet::from_type(&goal_type, false);
        
        // Try to find a combination that works
        for (_word, net) in &word_nets[0] {
            // Try to link this net to the goal
            // In a real implementation, we would try all possible combinations
            
            // For now, just convert the first one to a proof tree
            if let Some(proof_tree) = net.to_proof_tree() {
                return Some(proof_tree);
            }
        }
        
        // If no proof net is valid, fall back to natural deduction
        self.parse_with_natural_deduction(sentence)
    }
    
    /// Try to derive a proof for the sentence with the goal type
    fn prove_sentence(&self, axioms: &[ProofNode], goal: &LogicalType) -> Option<ProofNode> {
        // Queue for breadth-first search
        let mut queue = VecDeque::new();
        
        // Initial state: individual axioms
        queue.push_back(ProofSearchState::new(axioms.to_vec()));
        
        // BFS for derivation
        for _ in 0..self.config.max_depth {
            if queue.is_empty() {
                break;
            }
            
            let current_state = queue.pop_front().unwrap();
            
            // Check if this is a complete proof
            if current_state.is_complete(goal) {
                return current_state.get_proof();
            }
            
            // Try to apply logical rules to combine items
            for i in 0..current_state.items.len() {
                for j in 0..current_state.items.len() {
                    if i == j && self.config.strict_linear {
                        continue; // Skip same item (unless we allow contraction)
                    }
                    
                    // Try different rules based on the logic variant
                    let mut new_states = Vec::new();
                    
                    // Right implication elimination (function application)
                    match &current_state.items[i].logical_type {
                        LogicalType::RightImplication(a, b, _modality_i) => {
                            // Check if j matches the argument type
                            if self.types_match(a, &current_state.items[j].logical_type) {
                                // Apply the rule
                                let result_type = (**b).clone();
                                
                                let new_proof = ProofNode::infer(
                                    result_type,
                                    vec![current_state.items[i].clone(), current_state.items[j].clone()],
                                    "→E"
                                );
                                
                                let new_state = current_state.apply_rule(
                                    "→E",
                                    new_proof,
                                    vec![i, j]
                                );
                                
                                new_states.push(new_state);
                            }
                        },
                        LogicalType::LeftImplication(a, b, _modality_i) => {
                            // Check if j matches the argument type
                            if self.types_match(b, &current_state.items[j].logical_type) {
                                // Apply the rule
                                let result_type = (**a).clone();
                                
                                let new_proof = ProofNode::infer(
                                    result_type,
                                    vec![current_state.items[i].clone(), current_state.items[j].clone()],
                                    "←E"
                                );
                                
                                let new_state = current_state.apply_rule(
                                    "←E",
                                    new_proof,
                                    vec![i, j]
                                );
                                
                                new_states.push(new_state);
                            }
                        },
                        _ => {}
                    }
                    
                    // Apply product rules if enabled
                    if self.config.use_product {
                        // Product elimination
                        if let LogicalType::Product(a, b, _modality) = &current_state.items[i].logical_type {
                            // Create hypotheses for the components of the product
                            let hyp_a = ProofNode::axiom("x", (**a).clone());
                            let _hyp_b = ProofNode::axiom("y", (**b).clone());
                            
                            // This is a simplified implementation - in reality we'd need
                            // to track hypotheses and handle proper discharge
                            
                            let new_proof = ProofNode::infer(
                                LogicalType::s(), // Example goal
                                vec![
                                    hyp_a.clone(),
                                    current_state.items[i].clone(),
                                ],
                                "⊗E"
                            );
                            
                            let new_state = current_state.apply_rule(
                                "⊗E",
                                new_proof,
                                vec![i]
                            );
                            
                            new_states.push(new_state);
                        }
                    }
                    
                    // Apply modal rules if enabled
                    if self.config.use_modalities {
                        // Diamond elimination
                        if let LogicalType::Diamond(a, _modality) = &current_state.items[i].logical_type {
                            let hyp = ProofNode::axiom("x", (**a).clone());
                            
                            let new_proof = ProofNode::infer(
                                LogicalType::s(), // Example goal
                                vec![
                                    hyp.clone(),
                                    current_state.items[i].clone(),
                                ],
                                "◇E"
                            );
                            
                            let new_state = current_state.apply_rule(
                                "◇E",
                                new_proof,
                                vec![i]
                            );
                            
                            new_states.push(new_state);
                        }
                        
                        // Box elimination
                        if let LogicalType::Box(a, _modality) = &current_state.items[i].logical_type {
                            let new_proof = ProofNode::infer(
                                (**a).clone(),
                                vec![current_state.items[i].clone()],
                                "□E"
                            );
                            
                            let new_state = current_state.apply_rule(
                                "□E",
                                new_proof,
                                vec![i]
                            );
                            
                            new_states.push(new_state);
                        }
                    }
                    
                    // Apply displacement rules if enabled
                    if self.config.use_displacement {
                        // Up arrow elimination
                        if let LogicalType::UpArrow(a, b, index) = &current_state.items[i].logical_type {
                            if self.types_match(b, &current_state.items[j].logical_type) {
                                // Apply the rule
                                let result_type = (**a).clone();
                                
                                let new_proof = ProofNode::infer(
                                    result_type,
                                    vec![current_state.items[i].clone(), current_state.items[j].clone()],
                                    &format!("↑{}E", index)
                                );
                                
                                let new_state = current_state.apply_rule(
                                    &format!("↑{}E", index),
                                    new_proof,
                                    vec![i, j]
                                );
                                
                                new_states.push(new_state);
                            }
                        }
                        
                        // Down arrow elimination
                        if let LogicalType::DownArrow(a, b, index) = &current_state.items[i].logical_type {
                            if self.types_match(b, &current_state.items[j].logical_type) {
                                // Apply the rule
                                let result_type = (**a).clone();
                                
                                let new_proof = ProofNode::infer(
                                    result_type,
                                    vec![current_state.items[i].clone(), current_state.items[j].clone()],
                                    &format!("↓{}E", index)
                                );
                                
                                let new_state = current_state.apply_rule(
                                    &format!("↓{}E", index),
                                    new_proof,
                                    vec![i, j]
                                );
                                
                                new_states.push(new_state);
                            }
                        }
                    }
                    
                    // Add new states to the queue
                    for state in new_states {
                        queue.push_back(state);
                    }
                }
            }
        }
        
        // No proof found
        eprintln!("No valid proof found for sentence with goal type: {}", goal);
        None
    }
    
    /// Check if two types match, handling features if enabled
    fn types_match(&self, type1: &LogicalType, type2: &LogicalType) -> bool {
        if self.config.use_features {
            // Try unification
            if let Some(_) = type1.unify(type2) {
                return true;
            }
            false
        } else {
            // Simple equality
            type1 == type2
        }
    }
}

impl ParserTrait for TLGParser {
    type Cat = LogicalType;
    type Node = ProofNode;
    type Config = ParserConfig;
    
    fn parse(&self, sentence: &str) -> Option<Self::Node> {
        // If using proof nets, try that approach first
        if self.config.use_proof_nets {
            self.parse_with_proof_nets(sentence)
        } else {
            // Otherwise, use the traditional natural deduction approach
            self.parse_with_natural_deduction(sentence)
        }
    }
    
    fn add_to_lexicon(&mut self, word: &str, category: Self::Cat) {
        if self.validate_type(&category) {
            self.lexicon.add(word, category);
        } else {
            eprintln!("Warning: Invalid logical type for '{}'.", word);
        }
    }
    
    fn config(&self) -> &Self::Config {
        &self.config
    }
    
    fn set_config(&mut self, config: Self::Config) {
        self.config = config;
    }
    
    fn create_category_with_features(&self, cat_str: &str, features: &[(&str, &str)]) -> Result<Self::Cat, crate::common::error::Error> {
        // Create a feature structure from the provided features
        let mut feature_struct = FeatureStructure::new();
        
        for (feat_name, feat_value) in features {
            // Validate the feature name and value
            if !self.feature_registry.is_feature_registered(feat_name) {
                return Err(crate::common::error::Error::ParseError(
                    format!("Invalid feature: {}", feat_name)
                ));
            }
            
            if !self.feature_registry.is_value_valid(feat_name, feat_value) {
                return Err(crate::common::error::Error::ParseError(
                    format!("Invalid value '{}' for feature '{}'", feat_value, feat_name)
                ));
            }
            
            feature_struct.add(feat_name, FeatureValue::Atomic(feat_value.to_string()));
        }
        
        // Create the logical type with features
        if self.atomic_types.is_registered(cat_str) {
            Ok(LogicalType::atomic_with_features(cat_str, &feature_struct))
        } else {
            Err(crate::common::error::Error::ParseError(
                format!("Invalid category: {}", cat_str)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper to create a simple parser for testing
    fn setup_test_parser() -> TLGParser {
        let mut parser = TLGParser::new();
        
        // Set up a minimal lexicon for testing
        let s = LogicalType::s();
        let np = LogicalType::np();
        let n = LogicalType::n();
        
        parser.add_to_lexicon("the", LogicalType::left_impl(np.clone(), n.clone()));
        parser.add_to_lexicon("cat", n.clone());
        parser.add_to_lexicon("sleeps", LogicalType::left_impl(s.clone(), np.clone()));
        
        parser
    }
    
    #[test]
    fn test_basic_parsing() {
        let parser = setup_test_parser();
        
        // Test a simple sentence
        let result = parser.parse("the cat sleeps");
        assert!(result.is_some());
        
        // Test an invalid sentence
        let result = parser.parse("cat the sleeps");
        assert!(result.is_none());
    }
    
    #[test]
    fn test_with_features() {
        let mut parser = setup_test_parser();
        
        // Enable features
        let mut config = parser.config.clone();
        config.use_features = true;
        parser.config = config;
        
        // Add feature-rich lexical entries
        let mut sg_feat = FeatureStructure::new();
        sg_feat.add("num", FeatureValue::Atomic("sg".to_string()));
        
        let mut pl_feat = FeatureStructure::new();
        pl_feat.add("num", FeatureValue::Atomic("pl".to_string()));
        
        let n_sg = LogicalType::atomic_with_features("n", &sg_feat);
        let n_pl = LogicalType::atomic_with_features("n", &pl_feat);
        
        let np_sg = LogicalType::atomic_with_features("np", &sg_feat);
        let np_pl = LogicalType::atomic_with_features("np", &pl_feat);
        
        let s = LogicalType::s();
        
        parser.add_to_lexicon("cat", n_sg.clone());
        parser.add_to_lexicon("cats", n_pl.clone());
        
        parser.add_to_lexicon("a", LogicalType::left_impl(np_sg.clone(), n_sg.clone()));
        parser.add_to_lexicon("some", LogicalType::left_impl(np_pl.clone(), n_pl.clone()));
        
        parser.add_to_lexicon("sleeps", LogicalType::left_impl(s.clone(), np_sg.clone()));
        parser.add_to_lexicon("sleep", LogicalType::left_impl(s.clone(), np_pl.clone()));
        
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
    fn test_parser_config() {
        // Test default config
        let default_parser = TLGParser::new();
        assert_eq!(default_parser.config.logic_variant, "NL");
        assert!(default_parser.config.use_product);
        assert!(!default_parser.config.use_modalities);
        
        // Test custom config
        let custom_config = ParserConfig {
            use_modalities: true,
            use_displacement: true,
            logic_variant: "NL(◇↑)".to_string(),
            ..ParserConfig::default()
        };
        
        let custom_parser = TLGParser::with_config(custom_config);
        assert_eq!(custom_parser.config.logic_variant, "NL(◇↑)");
        assert!(custom_parser.config.use_modalities);
        assert!(custom_parser.config.use_displacement);
    }
    
    #[test]
    fn test_modal_parsing() {
        let mut parser = TLGParser::new();
        
        // Enable modalities
        let mut config = parser.config.clone();
        config.use_modalities = true;
        parser.config = config;
        
        // Register a modality
        parser.register_modality(1, vec![
            crate::tlg::logical_type::StructuralProperty::Associativity
        ]);
        
        // Create the modality
        let m1 = Modality::new(1);
        
        let s = LogicalType::s();
        let np = LogicalType::np();
        
        // Modal verb type
        let modal_verb = LogicalType::left_impl_with_modality(s.clone(), np.clone(), m1.clone());
        
        parser.add_to_lexicon("walks", modal_verb);
        parser.add_to_lexicon("John", np.clone());
        
        // Test parsing with modality
        let result = parser.parse("John walks");
        assert!(result.is_some());
    }
    
    #[test]
    fn test_displacement_parsing() {
        let mut parser = TLGParser::new();
        
        // Enable displacement calculus
        let mut config = parser.config.clone();
        config.use_displacement = true;
        parser.config = config;
        
        let s = LogicalType::s();
        let np = LogicalType::np();
        
        // Wh-extraction type
        let wh_type = LogicalType::up_arrow(s.clone(), np.clone(), 1);
        
        // Transitive verb
        let verb_type = LogicalType::left_impl(
            LogicalType::left_impl(s.clone(), np.clone()),
            np.clone()
        );
        
        parser.add_to_lexicon("what", wh_type);
        parser.add_to_lexicon("John", np.clone());
        parser.add_to_lexicon("sees", verb_type);
        
        // Test wh-question
        let result = parser.parse("what John sees");
        assert!(result.is_some());
    }
}