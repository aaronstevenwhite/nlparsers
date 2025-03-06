//! Parser for Head-Driven Phrase Structure Grammar
//!
//! This module provides a chart parser for HPSG, which uses the grammar rules
//! and principles to parse sentences.

use std::collections::{HashMap, HashSet};
use crate::common::{self, Parser as ParserTrait, ParseNode};
use crate::hpsg::feature_structure::{FeatureStructure, TypedValue, FeatureType};
use crate::hpsg::sign::{Sign, Category};
use crate::hpsg::rule::{Rule, RuleSchema};
use crate::hpsg::lexicon::Lexicon;
use crate::hpsg::principle::Principle;
use crate::hpsg::registry::HPSGRegistry;

/// Configuration for the HPSG parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum chart size
    pub max_chart_size: usize,
    /// Whether to use bottom-up parsing
    pub bottom_up: bool,
    /// Whether to use top-down filtering
    pub top_down_filter: bool,
    /// Maximum recursion depth
    pub max_depth: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_chart_size: 1000,
            bottom_up: true,
            top_down_filter: false,
            max_depth: 20,
        }
    }
}

/// A parse tree node for HPSG
#[derive(Debug, Clone)]
pub struct ParseTree {
    /// The sign at this node
    pub sign: Sign,
    /// Child nodes
    pub children: Vec<ParseTree>,
    /// Rule used to create this node (if not a leaf)
    pub rule_name: Option<String>,
}

impl ParseTree {
    /// Create a new parse tree node
    pub fn new(sign: Sign) -> Self {
        Self {
            sign,
            children: Vec::new(),
            rule_name: None,
        }
    }
    
    /// Create a parse tree node with children
    pub fn with_children(sign: Sign, children: Vec<ParseTree>) -> Self {
        Self {
            sign,
            children,
            rule_name: None,
        }
    }
    
    /// Create a parse tree node with children and rule name
    pub fn with_rule(sign: Sign, children: Vec<ParseTree>, rule_name: &str) -> Self {
        Self {
            sign,
            children,
            rule_name: Some(rule_name.to_string()),
        }
    }
    
    /// Get the surface string for this parse tree
    pub fn surface_string(&self) -> String {
        if self.children.is_empty() {
            self.sign.phonetic_form()
        } else {
            self.children.iter()
                .map(|child| child.surface_string())
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

// Implement ParseNode trait for ParseTree
impl ParseNode for ParseTree {
    type Cat = Category;
    
    fn category(&self) -> &Self::Cat {
        &self.sign.category
    }
    
    fn word(&self) -> Option<&str> {
        if self.is_leaf() {
            Some(&self.sign.phonetic_form())
        } else {
            None
        }
    }
    
    fn children(&self) -> &[Self] {
        &self.children
    }
    
    fn rule(&self) -> Option<&str> {
        self.rule_name.as_deref()
    }
    
    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
    
    fn node_features(&self) -> Option<&common::FeatureStructure> {
        None // We use HPSG-specific feature structures
    }
}

/// HPSG parser using chart parsing
#[derive(Debug)]
pub struct HPSGParser {
    /// Grammar rules
    rules: Vec<Rule>,
    /// Lexicon
    lexicon: Lexicon,
    /// Principles
    principles: Vec<Box<dyn Principle>>,
    /// Registry
    registry: HPSGRegistry,
    /// Configuration
    config: ParserConfig,
    /// Next ID for signs
    next_id: usize,
}

impl HPSGParser {
    /// Create a new HPSG parser
    pub fn new(rules: Vec<Rule>, lexicon: Lexicon, principles: Vec<Box<dyn Principle>>, registry: HPSGRegistry) -> Self {
        Self {
            rules,
            lexicon,
            principles,
            registry,
            config: ParserConfig::default(),
            next_id: 0,
        }
    }
    
    /// Get the next ID for a sign
    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    /// Parse a sentence using chart parsing
    pub fn parse_sentence(&mut self, sentence: &str) -> Option<ParseTree> {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        if words.is_empty() {
            return None;
        }
        
        // Initialize chart
        let mut chart = Chart::new(words.len());
        
        // Add lexical entries to the chart
        for (i, word) in words.iter().enumerate() {
            if let Some(entries) = self.lexicon.get_entries(word) {
                for entry in entries {
                    let sign = entry.to_sign(self.next_id());
                    chart.add_edge(i, i+1, sign);
                }
            } else {
                // Unknown word
                return None;
            }
        }
        
        // Apply rules until no new edges are added
        let mut added = true;
        while added {
            added = false;
            
            // Try to apply rules to existing edges
            let edges = chart.get_all_edges();
            for rule in &self.rules {
                for edge_set in edges.values() {
                    for edge in edge_set {
                        // Try to apply the rule to this edge
                        if let Some(new_edges) = self.apply_rule(rule, edge, &chart) {
                            for (start, end, sign) in new_edges {
                                if !chart.has_edge(start, end, &sign) {
                                    chart.add_edge(start, end, sign.clone());
                                    added = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Check if we have a complete parse
        if let Some(parses) = chart.get_edges(0, words.len()) {
            // Find a parse that spans the whole sentence
            for sign in parses {
                if sign.sign_type == "phrase" {
                    // Convert to parse tree
                    return Some(self.build_parse_tree(sign.clone(), &chart, 0, words.len()));
                }
            }
        }
        
        None
    }
    
    /// Apply a rule to an edge and return new edges
    fn apply_rule(&mut self, rule: &Rule, edge: &Sign, chart: &Chart) -> Option<Vec<(usize, usize, Sign)>> {
        // This is a simplified implementation
        // In a real parser, we would need to handle different rule schemas
        
        match rule.schema {
            RuleSchema::HeadComplement => {
                // Try to find complements for this head
                self.apply_binary_rule(rule, edge, chart)
            },
            RuleSchema::HeadSubject => {
                // Try to find a subject for this head
                self.apply_binary_rule(rule, edge, chart)
            },
            _ => None,
        }
    }
    
    /// Apply a binary rule (head + one other constituent)
    fn apply_binary_rule(&mut self, rule: &Rule, edge: &Sign, chart: &Chart) -> Option<Vec<(usize, usize, Sign)>> {
        let mut new_edges = Vec::new();
        
        // Get the span of this edge
        let (start, end) = chart.get_span(edge)?;
        
        // Look for potential other constituents
        // This is a simplified approach - in a real parser we would use feature constraints
        
        // Try combining with constituents to the left
        for left_start in 0..start {
            if let Some(left_edges) = chart.get_edges(left_start, start) {
                for left_edge in left_edges {
                    let daughters = vec![left_edge.clone(), edge.clone()];
                    if let Some(mother) = rule.apply(&daughters, self.next_id()) {
                        new_edges.push((left_start, end, mother));
                    }
                }
            }
        }
        
        // Try combining with constituents to the right
        for right_end in (end+1)..=chart.size() {
            if let Some(right_edges) = chart.get_edges(end, right_end) {
                for right_edge in right_edges {
                    let daughters = vec![edge.clone(), right_edge.clone()];
                    if let Some(mother) = rule.apply(&daughters, self.next_id()) {
                        new_edges.push((start, right_end, mother));
                    }
                }
            }
        }
        
        if new_edges.is_empty() {
            None
        } else {
            Some(new_edges)
        }
    }
    
    /// Build a parse tree from a chart
    fn build_parse_tree(&self, sign: Sign, chart: &Chart, start: usize, end: usize) -> ParseTree {
        // If this is a lexical item, it's a leaf node
        if start + 1 == end {
            return ParseTree::new(sign);
        }
        
        // Otherwise, find the daughters
        let mut children = Vec::new();
        let mut rule_name = None;
        
        // This is a simplified approach - in a real parser we would use the actual daughters
        // stored in the sign, but here we're reconstructing from the chart
        if let Some(daughters) = &sign.daughters {
            for daughter in daughters {
                // Find the span of this daughter
                if let Some((d_start, d_end)) = chart.get_span(daughter) {
                    children.push(self.build_parse_tree(daughter.clone(), chart, d_start, d_end));
                }
            }
            
            // Get the rule name if available
            rule_name = sign.rule_name.clone();
        }
        
        if let Some(rule) = rule_name {
            ParseTree::with_rule(sign, children, &rule)
        } else {
            ParseTree::with_children(sign, children)
        }
    }
}

// Implement the common Parser trait for our HPSG parser
impl ParserTrait for HPSGParser {
    type Cat = Category;
    type Node = ParseTree;
    type Config = ParserConfig;
    
    fn parse(&self, sentence: &str) -> Option<Self::Node> {
        // Create a mutable clone for parsing
        let mut parser = self.clone();
        parser.parse_sentence(sentence)
    }
    
    fn add_to_lexicon(&mut self, word: &str, category: Self::Cat) {
        // Create a feature structure from the category
        let fs = category.feature_structure.unwrap_or_else(|| {
            FeatureStructure::new(&category.name, self.next_id())
        });
        
        // Add to lexicon
        self.lexicon.add_entry(word, &category.name, &fs);
    }
    
    fn config(&self) -> &Self::Config {
        &self.config
    }
    
    fn set_config(&mut self, config: Self::Config) {
        self.config = config;
    }
    
    fn create_category_with_features(&self, name: &str, features: &[(&str, &str)]) -> Result<Self::Cat, common::Error> {
        let mut fs = FeatureStructure::new(name, 0);
        
        for (feat, value) in features {
            fs.set(feat, TypedValue {
                type_name: "atomic".to_string(),
                value: FeatureType::String(value.to_string()),
                id: fs.get_next_id(),
            });
        }
        
        Ok(Category::with_feature_structure(name, fs))
    }
}

// Implement Clone for Parser
impl Clone for HPSGParser {
    fn clone(&self) -> Self {
        // We need to manually clone the principles since Box<dyn Principle> doesn't implement Clone
        let principles = self.principles.iter()
            .map(|p| p.clone_box())
            .collect();
            
        Self {
            rules: self.rules.clone(),
            lexicon: self.lexicon.clone(),
            principles,
            registry: self.registry.clone(),
            config: self.config.clone(),
            next_id: self.next_id,
        }
    }
}

/// A chart for parsing
#[derive(Debug)]
struct Chart {
    /// Edges in the chart, indexed by (start, end)
    edges: HashMap<(usize, usize), Vec<Sign>>,
    /// Size of the input
    size: usize,
    /// Reverse index from signs to their spans
    spans: HashMap<usize, (usize, usize)>,
}

impl Chart {
    /// Create a new chart for an input of the given size
    fn new(size: usize) -> Self {
        Self {
            edges: HashMap::new(),
            size,
            spans: HashMap::new(),
        }
    }
    
    /// Add an edge to the chart
    fn add_edge(&mut self, start: usize, end: usize, sign: Sign) {
        let entry = self.edges.entry((start, end)).or_insert_with(Vec::new);
        
        // Store the span for this sign
        self.spans.insert(sign.index, (start, end));
        
        // Only add if not already present
        if !entry.iter().any(|s| s.index == sign.index) {
            entry.push(sign);
        }
    }
    
    /// Get edges spanning from start to end
    fn get_edges(&self, start: usize, end: usize) -> Option<&Vec<Sign>> {
        self.edges.get(&(start, end))
    }
    
    /// Get all edges in the chart
    fn get_all_edges(&self) -> &HashMap<(usize, usize), Vec<Sign>> {
        &self.edges
    }
    
    /// Check if the chart has an edge
    fn has_edge(&self, start: usize, end: usize, sign: &Sign) -> bool {
        if let Some(edges) = self.get_edges(start, end) {
            edges.iter().any(|s| s.index == sign.index)
        } else {
            false
        }
    }
    
    /// Get the span of a sign
    fn get_span(&self, sign: &Sign) -> Option<(usize, usize)> {
        self.spans.get(&sign.index).copied()
    }
    
    /// Get the size of the input
    fn size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hpsg::principle::{HeadFeaturePrinciple, ValencePrinciple};
    
    fn create_test_parser() -> HPSGParser {
        // Create a simple grammar for testing
        let rules = Vec::new();
        
        // Create principles
        let principles: Vec<Box<dyn Principle>> = vec![
            Box::new(HeadFeaturePrinciple::new()),
            Box::new(ValencePrinciple::new()),
        ];
        
        // Create lexicon
        let lexicon = Lexicon::new();
        
        // Create registry
        let registry = HPSGRegistry::standard();
        
        HPSGParser::new(rules, lexicon, principles, registry)
    }
    
    #[test]
    fn test_parser_creation() {
        let parser = create_test_parser();
        
        assert_eq!(parser.rules.len(), 0);
        assert!(parser.lexicon.is_empty());
        assert_eq!(parser.principles.len(), 2);
    }
    
    #[test]
    fn test_next_id() {
        let mut parser = create_test_parser();
        
        assert_eq!(parser.next_id(), 0);
        assert_eq!(parser.next_id(), 1);
        assert_eq!(parser.next_id(), 2);
    }
    
    #[test]
    fn test_chart_creation() {
        let chart = Chart::new(3);
        
        assert_eq!(chart.size(), 3);
        assert!(chart.edges.is_empty());
    }
    
    #[test]
    fn test_add_to_lexicon() {
        let mut parser = create_test_parser();
        let category = Category::new("adj");
        
        parser.add_to_lexicon("big", category);
        
        assert!(parser.lexicon.contains("big"));
    }
}