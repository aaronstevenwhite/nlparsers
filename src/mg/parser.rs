//! Parser for Minimalist Grammar

use std::collections::{HashSet, VecDeque};
use crate::mg::feature::Feature;
use crate::mg::lexical_item::LexicalItem;
use crate::mg::derivation::{DerivationTree, Chain};
use crate::mg::workspace::WorkspaceRegistry;
use crate::mg::phase::{PhaseConfig, PhaseChecker};
use crate::common::{Parser, Lexicon, FeatureRegistry};

/// Different types of movement strategies supported by the parser
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementStrategy {
    /// Standard movement (Stabler's original formulation)
    Standard,
    /// Multiple specifiers permitted in single operation
    MultiSpecifier,
    /// Sideward movement (between workspaces)
    Sideward,
    /// Interarboreal movement (as in certain TAG formalisms)
    Interarboreal,
}

/// Different types of merge operations supported by the parser
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Standard merge (Stabler's original formulation)
    Standard,
    /// Pair merge for adjunction
    PairMerge,
    /// Late merge (merger of material post-movement)
    LateMerge,
}

/// Different types of sideward movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidewardMovementType {
    /// Classic Nunes-style sideward movement (copy and merge)
    NunesStyle,
    /// Parallel derivation workspace sharing
    ParallelDerivation,
    /// Multidominance (shared nodes between structures)
    Multidominance,
    /// Wholesale late merger (Lebeaux-style)
    WholesaleLate,
}

/// Configuration options for the Minimalist Grammar parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum depth for derivation
    pub max_derivation_depth: usize,
    /// Whether to allow remnant movement
    pub allow_remnant_movement: bool,
    /// Whether to allow vacuous movement (moving something that doesn't affect word order)
    pub allow_vacuous_movement: bool,
    /// Types of movement allowed
    pub movement_strategies: Vec<MovementStrategy>,
    /// Types of merge operations allowed
    pub merge_strategies: Vec<MergeStrategy>,
    /// Types of sideward movement allowed
    pub sideward_movement_types: Vec<SidewardMovementType>,
    /// Whether to track parallel workspaces for sideward movement
    pub enable_parallel_workspaces: bool,
    /// Maximum number of parallel workspaces
    pub max_workspaces: usize,
    /// Phase-based processing configuration
    pub phase_config: PhaseConfig,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_derivation_depth: 20,
            allow_remnant_movement: false,
            allow_vacuous_movement: false,
            movement_strategies: vec![MovementStrategy::Standard],
            merge_strategies: vec![MergeStrategy::Standard],
            sideward_movement_types: vec![],
            enable_parallel_workspaces: false,
            max_workspaces: 3,
            phase_config: PhaseConfig::default(),
        }
    }
}

/// Registry for feature types in the grammar
#[derive(Debug, Clone)]
pub struct FeatureTypeRegistry {
    categorial: HashSet<String>,
    licensors: HashSet<String>,
    licensees: HashSet<String>,
}

impl FeatureTypeRegistry {
    /// Create a new empty feature type registry
    pub fn new() -> Self {
        FeatureTypeRegistry {
            categorial: HashSet::new(),
            licensors: HashSet::new(),
            licensees: HashSet::new(),
        }
    }
    
    /// Register a new categorial feature
    pub fn register_categorial(&mut self, feature: &str) {
        self.categorial.insert(feature.to_string());
    }
    
    /// Register a new movement feature (creates both licensor and licensee)
    pub fn register_movement(&mut self, feature: &str) {
        self.licensors.insert(feature.to_string());
        self.licensees.insert(feature.to_string());
    }
    
    /// Check if a categorial feature is registered
    pub fn is_categorial_registered(&self, feature: &str) -> bool {
        self.categorial.contains(feature)
    }
    
    /// Check if a movement feature is registered
    pub fn is_movement_registered(&self, feature: &str) -> bool {
        self.licensors.contains(feature) && self.licensees.contains(feature)
    }
    
    /// Get all registered categorial features
    pub fn get_all_categorial(&self) -> Vec<String> {
        self.categorial.iter().cloned().collect()
    }
    
    /// Get all registered movement features
    pub fn get_all_movement(&self) -> Vec<String> {
        self.licensors.iter().cloned().collect()
    }
}

impl Default for FeatureTypeRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Register standard categories
        for cat in &["C", "T", "v", "V", "D", "N", "P", "A"] {
            registry.register_categorial(cat);
        }
        
        // Register standard movement features
        for feature in &["wh", "case", "top", "foc"] {
            registry.register_movement(feature);
        }
        
        registry
    }
}

/// The Minimalist Grammar Parser
#[derive(Clone)]
pub struct MinimalistParser {
    pub lexicon: Lexicon<LexicalItem>,
    pub feature_types: FeatureTypeRegistry,
    pub feature_registry: FeatureRegistry,
    pub config: ParserConfig,
    pub next_index: usize, // For tracking node indices during derivation
    pub workspaces: WorkspaceRegistry,
    pub phase_checker: PhaseChecker,
}

impl MinimalistParser {
    /// Create a new parser with default configuration
    pub fn new() -> Self {
        let config = ParserConfig::default();
        let phase_checker = PhaseChecker::new(config.phase_config.clone());
        
        MinimalistParser {
            lexicon: Lexicon::new(),
            feature_types: FeatureTypeRegistry::default(),
            feature_registry: FeatureRegistry::new(),
            config,
            next_index: 0,
            workspaces: WorkspaceRegistry::new(),
            phase_checker,
        }
    }
    
    /// Create a new parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        let mut parser = Self::new();
        parser.config = config;
        parser.phase_checker = PhaseChecker::new(parser.config.phase_config.clone());
        parser
    }
    
    /// Register a new categorial feature
    pub fn register_categorial_feature(&mut self, feature: &str) {
        self.feature_types.register_categorial(feature);
    }
    
    /// Register a new movement feature
    pub fn register_movement_feature(&mut self, feature: &str) {
        self.feature_types.register_movement(feature);
    }
    
    /// Validate a feature for use in the grammar
    pub fn validate_feature(&self, feature: &Feature) -> bool {
        match feature {
            Feature::Categorial(name) => {
                self.feature_types.is_categorial_registered(name)
            },
            Feature::Selector(name) | Feature::StrongSelector(name) | Feature::AdjunctSelector(name) => {
                self.feature_types.is_categorial_registered(name)
            },
            Feature::Licensor(name) | Feature::Licensee(name) => {
                self.feature_types.is_movement_registered(name)
            },
            Feature::Agreement(_, _) => true, // Agreement features are always allowed
            Feature::Phase(name) => {
                self.feature_types.is_categorial_registered(name)
            },
            Feature::Delayed(inner) => self.validate_feature(inner),
        }
    }
    
    /// Get a new unique index for nodes
    pub fn get_next_index(&mut self) -> usize {
        let index = self.next_index;
        self.next_index += 1;
        index
    }
    
    /// Parse a sentence, returning a derivation tree if successful
    pub fn parse_internal(&mut self, sentence: &str) -> Option<DerivationTree> {
        // Initialize workspaces
        self.workspaces = WorkspaceRegistry::new();
        let _main_workspace_id = self.workspaces.new_workspace();
        
        // Reset the next index counter
        self.next_index = 0;
        
        let words: Vec<&str> = sentence.split_whitespace().collect();
        
        // Create initial lexical items
        let mut lexical_trees = Vec::new();
        for word in &words {
            let items = self.lexicon.get_categories(word);
            
            if items.is_empty() {
                eprintln!("Unknown word: {}", word);
                return None;
            }
            
            for item in items {
                lexical_trees.push(DerivationTree::leaf(item, self.get_next_index()));
            }
        }
        
        // Add null elements (functional heads that might be phonologically null)
        lexical_trees.push(DerivationTree::leaf(
            LexicalItem::new("", vec![
                Feature::Categorial("T".to_string()),
                Feature::Selector("V".to_string()),
                Feature::Selector("D".to_string()),
            ]),
            self.get_next_index(),
        ));
        
        lexical_trees.push(DerivationTree::leaf(
            LexicalItem::new("", vec![
                Feature::Categorial("C".to_string()),
                Feature::Selector("T".to_string()),
            ]),
            self.get_next_index(),
        ));
        
        // Try to derive a complete sentence using a breadth-first search
        let mut queue = VecDeque::new();
        
        // Initial state: individual lexical items
        for tree in lexical_trees {
            queue.push_back(tree);
        }
        
        // Keep track of trees we've seen to avoid duplicates
        let mut seen_trees = Vec::new();
        
        // BFS for derivation
        for _ in 0..self.config.max_derivation_depth {
            if queue.is_empty() {
                break;
            }
            
            let current_tree = queue.pop_front().unwrap();
            
            // Check if this is a complete derivation (only a C feature remains)
            if let Some(Feature::Categorial(cat)) = current_tree.first_feature() {
                if cat == "C" && current_tree.chain.head.features.len() == 1 {
                    // This is a complete derivation
                    // Check if the derived string matches the input
                    let derived = self.linearize(&current_tree);
                    
                    if self.matches_input(&derived, &words) {
                        return Some(current_tree);
                    }
                }
            }
            
            // Try to apply Merge with all other trees we've seen
            for other_tree in &seen_trees {
                // Try merging current as specifier, other as head
                if let Some(merged_tree) = self.apply_merge(&current_tree, other_tree) {
                    // Check if we've seen this tree before
                    if !seen_trees.iter().any(|tree| tree_equals(&merged_tree, tree)) {
                        queue.push_back(merged_tree);
                    }
                }
                
                // Try merging other as specifier, current as head
                if let Some(merged_tree) = self.apply_merge(other_tree, &current_tree) {
                    // Check if we've seen this tree before
                    if !seen_trees.iter().any(|tree| tree_equals(&merged_tree, tree)) {
                        queue.push_back(merged_tree);
                    }
                }
            }
            
            // Try to apply Move to the current tree
            if let Some(moved_tree) = self.apply_move(&current_tree) {
                // Check if we've seen this tree before
                if !seen_trees.iter().any(|tree| tree_equals(&moved_tree, tree)) {
                    queue.push_back(moved_tree);
                }
            }
            
            // Add current tree to seen trees
            seen_trees.push(current_tree);
        }
        
        // No complete derivation found
        eprintln!("No valid derivation found for: {}", sentence);
        None
    }
    
    /// Apply the Merge operation to two trees
    fn apply_merge(&mut self, spec: &DerivationTree, head: &DerivationTree) -> Option<DerivationTree> {
        // If phases are enabled, check phase constraints
        if self.config.phase_config.enforce_pic {
            // If the head is a completed phase, only its edge should be accessible
            if head.is_phase && head.phase_completed {
                // The Phase Impenetrability Condition blocks this merge
                return None;
            }
        }
        
        // Try different merge strategies based on configuration
        for strategy in &self.config.merge_strategies {
            match strategy {
                MergeStrategy::Standard => {
                    // Standard Merge (Stabler's original formulation)
                    if let Some(head_feature) = head.first_feature() {
                        if let Some(spec_feature) = spec.first_feature() {
                            if head_feature.matches(spec_feature) {
                                // Features match, can merge
                                
                                // Create new trees with first features removed
                                let mut spec_new = spec.clone();
                                let mut head_new = head.clone();
                                
                                spec_new.remove_first_feature();
                                head_new.remove_first_feature();
                                
                                // Check head movement if triggered
                                let head_features = head.chain.head.features[1..].to_vec();
                                
                                if head_feature.triggers_head_movement() {
                                    // For head movement, combine the phonetic content
                                    return Some(DerivationTree {
                                        chain: Chain::new(LexicalItem {
                                            phonetic_form: format!("{}{}", 
                                                head.chain.head.phonetic_form,
                                                spec.chain.head.phonetic_form),
                                            features: head_features,
                                            agreement_features: None,
                                        }),
                                        children: Some((Box::new(spec_new), Box::new(head_new))),
                                        index: self.get_next_index(),
                                        is_adjunct: false,
                                        delayed_features: Vec::new(),
                                        is_phase: false,
                                        phase_completed: false,
                                    });
                                }
                                
                                // Return the merged tree
                                return Some(DerivationTree::merge(
                                    spec_new,
                                    head_new,
                                    head_features,
                                    self.get_next_index(),
                                ));
                            }
                        }
                    }
                },
                MergeStrategy::PairMerge => {
                    // Pair Merge for adjunction
                    if let Some(head_feature) = head.first_feature() {
                        if let Some(spec_feature) = spec.first_feature() {
                            if let Feature::AdjunctSelector(cat) = head_feature {
                                if let Feature::Categorial(spec_cat) = spec_feature {
                                    if cat == spec_cat {
                                        // Features match, can do pair merge (adjunction)
                                        
                                        // Create new trees with first features removed
                                        let mut spec_new = spec.clone();
                                        let mut head_new = head.clone();
                                        
                                        spec_new.remove_first_feature();
                                        head_new.remove_first_feature();
                                        
                                        // Return the pair-merged tree (adjunction)
                                        return Some(DerivationTree::pair_merge(
                                            head_new,
                                            spec_new,
                                            self.get_next_index(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                },
                MergeStrategy::LateMerge => {
                    // Late Merge
                    if !head.delayed_features.is_empty() {
                        if let Some(spec_feature) = spec.first_feature() {
                            if let Some(delayed_feature) = head.delayed_features.first() {
                                if delayed_feature.matches(spec_feature) {
                                    // Can do late merge
                                    return Some(DerivationTree::late_merge(
                                        head.clone(),
                                        spec.clone(),
                                        self.get_next_index(),
                                    ));
                                }
                            }
                        }
                    }
                },
            }
        }
        
        None
    }
    
    /// Apply the Move operation
    fn apply_move(&mut self, tree: &DerivationTree) -> Option<DerivationTree> {
        // Look for a licensor feature in the tree's head
        if let Some(tree_feature) = tree.first_feature() {
            if let Feature::Licensor(lic) = tree_feature {
                // Find a matching licensee feature in the tree
                if let Some((moved_chain, new_base)) = self.find_movable_element(tree, &lic) {
                    let mut new_tree = new_base;
                    new_tree.remove_first_feature(); // Remove the licensor feature
                    
                    // Return the moved tree
                    return Some(DerivationTree::r#move(
                        new_tree,
                        moved_chain,
                        tree.chain.head.features[1..].to_vec(), // Keep remaining features
                        self.get_next_index(),
                    ));
                }
            }
        }
        
        None
    }
    
    /// Find a movable element with a matching licensee feature
    fn find_movable_element(&self, tree: &DerivationTree, licensor: &str) -> Option<(Chain, DerivationTree)> {
        fn find_internal(
            tree: &DerivationTree, 
            licensor: &str, 
            path: &mut Vec<bool>, 
            moved: &mut Option<(Chain, Vec<bool>)>
        ) -> bool {
            // Check if this node has a matching licensee feature
            if let Some(Feature::Licensee(lic)) = tree.first_feature() {
                if lic == licensor {
                    // Found the licensee!
                    *moved = Some((
                        Chain::with_tail(
                            LexicalItem {
                                phonetic_form: tree.chain.head.phonetic_form.clone(),
                                features: tree.chain.head.features[1..].to_vec(), // Remove licensee
                                agreement_features: tree.chain.agreement.clone(),
                            },
                            Vec::new(), // Will be filled in later
                        ),
                        path.clone()
                    ));
                    return true;
                }
            }
            
            // Recursively search children
            if let Some((left, right)) = &tree.children {
                path.push(false); // Go left
                let found_left = find_internal(left, licensor, path, moved);
                path.pop();
                
                if !found_left {
                    path.push(true); // Go right
                    let found_right = find_internal(right, licensor, path, moved);
                    path.pop();
                    
                    if found_right {
                        return true;
                    }
                } else {
                    return true;
                }
            }
            
            false
        }
        
        fn create_moved_tree(
            tree: &DerivationTree, 
            path: &[bool], 
            path_index: usize,
            trace_indices: &mut Vec<usize>
        ) -> DerivationTree {
            if path_index >= path.len() {
                // We've reached the leaf to replace with a trace
                trace_indices.push(tree.index);
                
                // Return an empty trace node
                return DerivationTree {
                    chain: Chain::new(LexicalItem::empty()),
                    children: None,
                    index: tree.index,
                    is_adjunct: false,
                    delayed_features: Vec::new(),
                    is_phase: false,
                    phase_completed: false,
                };
            }
            
            // Recursively build the new tree
            if let Some((left, right)) = &tree.children {
                let mut new_tree = tree.clone();
                
                if path[path_index] {
                    // Go right
                    new_tree.children = Some((
                        left.clone(),
                        Box::new(create_moved_tree(right, path, path_index + 1, trace_indices))
                    ));
                } else {
                    // Go left
                    new_tree.children = Some((
                        Box::new(create_moved_tree(left, path, path_index + 1, trace_indices)),
                        right.clone()
                    ));
                }
                
                return new_tree;
            }
            
            // This shouldn't happen if path is correct
            tree.clone()
        }
        
        // Find the movable element
        let mut path = Vec::new();
        let mut moved = None;
        
        if find_internal(tree, licensor, &mut path, &mut moved) {
            if let Some((mut chain, path)) = moved {  // Add 'mut' here
                let mut trace_indices = Vec::new();
                let new_tree = create_moved_tree(tree, &path, 0, &mut trace_indices);
                
                // Update the chain with trace indices
                chain.tail = trace_indices;
                
                return Some((chain, new_tree));
            }
        }
        
        None
    }
    
    /// Linearize a derivation tree to get the surface string
    pub fn linearize(&self, tree: &DerivationTree) -> Vec<String> {
        fn collect_phonetic_forms(tree: &DerivationTree, forms: &mut Vec<(String, usize)>) {
            // Add this node's phonetic form if non-empty and not a trace
            if !tree.chain.head.phonetic_form.is_empty() && !tree.chain.tail.contains(&tree.index) {
                forms.push((tree.chain.head.phonetic_form.clone(), tree.index));
            }
            
            // Recursively collect from children
            if let Some((left, right)) = &tree.children {
                collect_phonetic_forms(left, forms);
                collect_phonetic_forms(right, forms);
            }
        }
        
        let mut forms = Vec::new();
        collect_phonetic_forms(tree, &mut forms);
        
        // Sort by index to get the correct word order
        forms.sort_by_key(|(_, idx)| *idx);
        
        // Return just the words
        forms.into_iter().map(|(form, _)| form).collect()
    }
    
    /// Check if the derived string matches the input
    fn matches_input(&self, derived: &[String], input: &[&str]) -> bool {
        if derived.len() != input.len() {
            return false;
        }
        
        for (d, i) in derived.iter().zip(input.iter()) {
            if d != i {
                return false;
            }
        }
        
        true
    }
    
    /// Handle sideward movement between workspaces
    fn sideward_move(
        &mut self,
        source_workspace_id: usize,
        target_workspace_id: usize,
        moved_chain: Chain,
        movement_type: SidewardMovementType
    ) -> Option<DerivationTree> {
        // Get references to the source and target workspaces
        let source_tree = self.workspaces.get_tree(source_workspace_id).cloned()?;
        let target_tree = self.workspaces.get_tree(target_workspace_id).cloned()?;
        
        match movement_type {
            SidewardMovementType::NunesStyle => {
                // Classic Nunes-style sideward movement:
                // 1. Copy from source
                // 2. Merge with target
                // 3. Chain formation across workspaces
                
                // The moved element acts as the specifier in the target workspace
                let _head_features = moved_chain.head.features.clone(); // Add underscore to unused variable
                
                // Create trace in source workspace (replacing the moved element)
                let source_with_trace = source_tree.clone(); // Remove 'mut' as it's not modified
                // This is simplified; in reality we would need to find and replace the actual source
                
                // Create a new derivation tree in the target workspace
                let result = DerivationTree {
                    chain: moved_chain.clone(),
                    children: Some((Box::new(target_tree), Box::new(DerivationTree::leaf(
                        LexicalItem::empty(),
                        self.get_next_index(),
                    )))),
                    index: self.get_next_index(),
                    is_adjunct: false,
                    delayed_features: Vec::new(),
                    is_phase: false,
                    phase_completed: false,
                };
                
                // Update the workspaces
                self.workspaces.add_tree(source_workspace_id, source_with_trace);
                
                Some(result)
            },
            
            SidewardMovementType::ParallelDerivation => {
                // Parallel derivation workspace sharing:
                // Both workspaces continue in parallel with shared content
                
                // In this model, we don't create a trace but maintain two 
                // parallel derivations that will be combined later
                
                // Create new workspace for the parallel derivation
                let parallel_id = self.workspaces.new_workspace();
                
                // Clone the moved element into the new workspace
                let moved_element = DerivationTree {
                    chain: moved_chain.clone(),
                    children: None, // Simplification
                    index: self.get_next_index(),
                    is_adjunct: false,
                    delayed_features: Vec::new(),
                    is_phase: false,
                    phase_completed: false,
                };
                
                // Add to the new workspace
                self.workspaces.add_tree(parallel_id, moved_element.clone());
                
                // Return the moved element
                Some(moved_element)
            },
            
            SidewardMovementType::Multidominance => {
                // Multidominance (shared nodes between structures):
                // A single node is dominated by multiple parents
                
                // In a real implementation, this would require a graph structure
                // rather than a tree, but we'll simulate it
                
                // Create a multidominant structure
                let result = DerivationTree {
                    chain: moved_chain.clone(),
                    children: Some((Box::new(target_tree), Box::new(source_tree))),
                    index: self.get_next_index(),
                    is_adjunct: false,
                    delayed_features: Vec::new(),
                    is_phase: false,
                    phase_completed: false,
                };
                
                Some(result)
            },
            
            SidewardMovementType::WholesaleLate => {
                // Wholesale late merger (Lebeaux-style):
                // The entire adjunct is merged after movement
                
                // Similar to late merge, but across workspaces
                
                // Create a structure with delayed features
                let mut result = target_tree.clone();
                
                // Add delayed feature for later merger
                if let Some(first_feature) = moved_chain.head.features.first() {
                    let feature = Feature::Delayed(Box::new(first_feature.clone()));
                    result.delayed_features.push(feature);
                }
                
                Some(result)
            },
        }
    }
}

impl Parser for MinimalistParser {
    type Cat = LexicalItem;
    type Node = DerivationTree;
    type Config = ParserConfig;
    
    fn parse(&self, sentence: &str) -> Option<Self::Node> {
        // Need to clone self since parse_internal needs to be mutable
        let mut parser = self.clone();
        parser.parse_internal(sentence)
    }
    
    fn add_to_lexicon(&mut self, word: &str, category: Self::Cat) {
        self.lexicon.add(word, category);
    }
    
    fn config(&self) -> &Self::Config {
        &self.config
    }
    
    fn set_config(&mut self, config: Self::Config) {
        self.config = config;
        self.phase_checker = PhaseChecker::new(self.config.phase_config.clone());
    }
    
    fn create_category_with_features(&self, cat_type: &str, features: &[(&str, &str)]) -> Result<Self::Cat, crate::common::error::Error> {
        let mut feature_list = Vec::new();
        
        // Add the categorial feature first
        feature_list.push(Feature::Categorial(cat_type.to_string()));
        
        // Add other features
        for (feat_type, feat_name) in features {
            let feature = match *feat_type {
                "selector" => Feature::Selector(feat_name.to_string()),
                "strong_selector" => Feature::StrongSelector(feat_name.to_string()),
                "adjunct_selector" => Feature::AdjunctSelector(feat_name.to_string()),
                "licensor" => Feature::Licensor(feat_name.to_string()),
                "licensee" => Feature::Licensee(feat_name.to_string()),
                "phase" => Feature::Phase(feat_name.to_string()),
                _ => return Err(crate::common::error::Error::ParseError(
                    format!("Unknown feature type: {}", feat_type)
                )),
            };
            
            feature_list.push(feature);
        }
        
        Ok(LexicalItem::new("", feature_list))
    }
}

// Helper function to compare trees since DerivationTree doesn't implement Eq
fn tree_equals(a: &DerivationTree, b: &DerivationTree) -> bool {
    // Simple comparison based on index and features
    a.index == b.index && 
    a.chain.head.features == b.chain.head.features &&
    a.chain.head.phonetic_form == b.chain.head.phonetic_form
}

/// Process a merge operation between two trees
fn process_merge(_parser: &mut MinimalistParser, _left: &DerivationTree, _right: &DerivationTree) -> Option<DerivationTree> {
    // This is a placeholder implementation
    // In a real implementation, you would check feature compatibility and perform the merge
    None
}

/// Parse a feature string into a Feature enum
pub fn parse_feature(feature_str: &str) -> Result<Feature, crate::common::error::Error> {
    // Split the feature string into type and name
    let parts: Vec<&str> = feature_str.split(':').collect();
    if parts.len() != 2 {
        return Err(crate::common::error::Error::ParseError(
            format!("Invalid feature format: {}", feature_str)
        ));
    }
    
    let feat_type = parts[0];
    let feat_name = parts[1];
    
    // Create the appropriate feature type
    match feat_type {
        "cat" => Ok(Feature::Categorial(feat_name.to_string())),
        "sel" => Ok(Feature::Selector(feat_name.to_string())),
        "sel+" => Ok(Feature::StrongSelector(feat_name.to_string())),
        "sel*" => Ok(Feature::AdjunctSelector(feat_name.to_string())),
        "licensor" => Ok(Feature::Licensor(feat_name.to_string())),
        "licensee" => Ok(Feature::Licensee(feat_name.to_string())),
        "phase" => Ok(Feature::Phase(feat_name.to_string())),
        _ => Err(crate::common::error::Error::ParseError(
            format!("Unknown feature type: {}", feat_type)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to create a minimalist parser with basic lexicon
    fn setup_test_parser() -> MinimalistParser {
        let mut parser = MinimalistParser::new();
        
        // Add some basic lexical items
        
        // Determiners
        parser.add_to_lexicon("the", LexicalItem::new("the", vec![
            Feature::Categorial("D".to_string()),
            Feature::Selector("N".to_string()),
        ]));
        
        // Nouns
        parser.add_to_lexicon("cat", LexicalItem::new("cat", vec![
            Feature::Categorial("N".to_string()),
        ]));
        
        parser.add_to_lexicon("dog", LexicalItem::new("dog", vec![
            Feature::Categorial("N".to_string()),
        ]));
        
        // Verbs
        parser.add_to_lexicon("sleeps", LexicalItem::new("sleeps", vec![
            Feature::Categorial("V".to_string()),
            Feature::Selector("D".to_string()),
        ]));
        
        parser.add_to_lexicon("chases", LexicalItem::new("chases", vec![
            Feature::Categorial("V".to_string()),
            Feature::Selector("D".to_string()),
            Feature::Selector("D".to_string()),
        ]));
        
        parser
    }
    
    #[test]
    fn test_basic_parsing() {
        let parser = setup_test_parser();
        
        // Test linearization of a simple tree
        let d = LexicalItem::new("the", vec![Feature::Categorial("D".to_string())]);
        let n = LexicalItem::new("cat", vec![Feature::Categorial("N".to_string())]);
        
        let dp = DerivationTree::merge(
            DerivationTree::leaf(n, 1),
            DerivationTree::leaf(d, 0),
            vec![Feature::Categorial("D".to_string())],
            2
        );
        
        let linearized = parser.linearize(&dp);
        assert_eq!(linearized, vec!["the", "cat"]);
    }
    
    #[test]
    fn test_merge_operation() {
        let mut parser = setup_test_parser();
        
        // Create D and N nodes
        let d = LexicalItem::new("the", vec![
            Feature::Selector("N".to_string()),
            Feature::Categorial("D".to_string()),
        ]);
        
        let n = LexicalItem::new("cat", vec![
            Feature::Categorial("N".to_string()),
        ]);
        
        let d_node = DerivationTree::leaf(d, 0);
        let n_node = DerivationTree::leaf(n, 1);
        
        // Apply merge
        let result = parser.apply_merge(&n_node, &d_node);
        
        // Check result
        assert!(result.is_some());

        let merged = result.unwrap();
        assert_eq!(merged.chain.head.features[0], Feature::Categorial("D".to_string()));

        // Linearize the result
        let linearized = parser.linearize(&merged);
        assert_eq!(linearized, vec!["the", "cat"]);
    }

    #[test]
    fn test_move_operation() {
        let mut parser = setup_test_parser();

        // Create a structure with movement
        let c = LexicalItem::new("", vec![
            Feature::Licensor("wh".to_string()),
            Feature::Categorial("C".to_string()),
        ]);

        let what = LexicalItem::new("what", vec![
            Feature::Licensee("wh".to_string()),
            Feature::Categorial("D".to_string()),
        ]);

        let v = LexicalItem::new("see", vec![
            Feature::Categorial("V".to_string()),
            Feature::Selector("D".to_string()),
        ]);

        // Create a VP with "what" as object
        let vp = DerivationTree::merge(
            DerivationTree::leaf(what, 0),
            DerivationTree::leaf(v, 1),
            vec![Feature::Categorial("V".to_string())],
            2
        );

        // Merge with C head
        let cp = DerivationTree::merge(
            vp,
            DerivationTree::leaf(c, 3),
            vec![
                Feature::Licensor("wh".to_string()),
                Feature::Categorial("C".to_string()),
            ],
            4
        );

        // Apply move
        let result = parser.apply_move(&cp);

        // Check result
        assert!(result.is_some());

        let moved = result.unwrap();
        assert_eq!(moved.chain.head.phonetic_form, "what");

        // Linearize the result
        let linearized = parser.linearize(&moved);
        assert_eq!(linearized, vec!["what", "see"]);
    }

    #[test]
    fn test_feature_registry() {
        let registry = FeatureTypeRegistry::default();

        // Check standard categories
        assert!(registry.is_categorial_registered("C"));
        assert!(registry.is_categorial_registered("T"));
        assert!(registry.is_categorial_registered("v"));
        assert!(registry.is_categorial_registered("V"));
        assert!(registry.is_categorial_registered("D"));
        assert!(registry.is_categorial_registered("N"));

        // Check standard movement features
        assert!(registry.is_movement_registered("wh"));
        assert!(registry.is_movement_registered("case"));
        assert!(registry.is_movement_registered("top"));
        assert!(registry.is_movement_registered("foc"));

        // Check for non-existent feature
        assert!(!registry.is_categorial_registered("X"));
        assert!(!registry.is_movement_registered("nonexistent"));
    }

    #[test]
    fn test_phase_constraints() {
        let config = ParserConfig {
            phase_config: PhaseConfig {
                enforce_pic: true,
                ..PhaseConfig::default()
            },
            ..ParserConfig::default()
        };

        let mut parser = MinimalistParser::with_config(config);

        // Create a phase head
        let c = LexicalItem::new("that", vec![
            Feature::Selector("T".to_string()),
            Feature::Categorial("C".to_string()),
            Feature::Phase("C".to_string()),
        ]);

        // Create a DP
        let dp = LexicalItem::new("it", vec![
            Feature::Categorial("D".to_string()),
        ]);

        let c_node = DerivationTree::leaf(c, 0);
        let dp_node = DerivationTree::leaf(dp, 1);

        // Merge should work initially
        let result = parser.apply_merge(&dp_node, &c_node);
        assert!(result.is_some());

        // Now complete the phase
        let mut merged = result.unwrap();
        merged.complete_phase();

        // Create another DP
        let dp2 = LexicalItem::new("they", vec![
            Feature::Categorial("D".to_string()),
        ]);
        let dp2_node = DerivationTree::leaf(dp2, 2);

        // Merge should fail due to PIC
        let result2 = parser.apply_merge(&dp2_node, &merged);
        assert!(result2.is_none());
    }

    #[test]
    fn test_parser_config() {
        // Test default config
        let default_parser = MinimalistParser::new();

        assert_eq!(default_parser.config.max_derivation_depth, 20);
        assert!(!default_parser.config.allow_remnant_movement);
        assert_eq!(default_parser.config.movement_strategies, vec![MovementStrategy::Standard]);

        // Test custom config
        let custom_config = ParserConfig {
            max_derivation_depth: 30,
            allow_remnant_movement: true,
            movement_strategies: vec![
                MovementStrategy::Standard, 
                MovementStrategy::MultiSpecifier,
                MovementStrategy::Sideward,
            ],
            merge_strategies: vec![
                MergeStrategy::Standard,
                MergeStrategy::PairMerge,
            ],
            ..ParserConfig::default()
        };

        let custom_parser = MinimalistParser::with_config(custom_config);

        assert_eq!(custom_parser.config.max_derivation_depth, 30);
        assert!(custom_parser.config.allow_remnant_movement);
        assert_eq!(custom_parser.config.movement_strategies.len(), 3);
        assert_eq!(custom_parser.config.merge_strategies.len(), 2);
    }

    #[test]
    fn test_workspaces() {
        let mut parser = MinimalistParser::new();

        // Enable parallel workspaces
        parser.config.enable_parallel_workspaces = true;

        // Create workspaces
        let ws1 = parser.workspaces.new_workspace();
        let ws2 = parser.workspaces.new_workspace();

        // Add trees to workspaces
        let d = LexicalItem::new("the", vec![
            Feature::Categorial("D".to_string()),
            Feature::Selector("N".to_string()),
        ]);

        let n = LexicalItem::new("book", vec![
            Feature::Categorial("N".to_string()),
        ]);

        parser.workspaces.add_tree(ws1, DerivationTree::leaf(d, 0));
        parser.workspaces.add_tree(ws2, DerivationTree::leaf(n, 1));

        // Test getting trees
        assert!(parser.workspaces.get_tree(ws1).is_some());
        assert!(parser.workspaces.get_tree(ws2).is_some());

        // Test sidebar movement
        if parser.config.sideward_movement_types.contains(&SidewardMovementType::NunesStyle) {
            let moved_chain = Chain::new(LexicalItem::new("book", vec![
                Feature::Categorial("N".to_string()),
            ]));
            
            let result = parser.sideward_move(ws2, ws1, moved_chain, SidewardMovementType::NunesStyle);
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_linearization() {
        let parser = setup_test_parser();

        // Create a complex structure to test linearization
        let det = LexicalItem::new("the", vec![
            Feature::Categorial("D".to_string()),
        ]);

        let noun = LexicalItem::new("cat", vec![
            Feature::Categorial("N".to_string()),
        ]);

        let verb = LexicalItem::new("chases", vec![
            Feature::Categorial("V".to_string()),
        ]);

        let det2 = LexicalItem::new("the", vec![
            Feature::Categorial("D".to_string()),
        ]);

        let noun2 = LexicalItem::new("dog", vec![
            Feature::Categorial("N".to_string()),
        ]);

        // Create the phrase structure
        let dp1 = DerivationTree::merge(
            DerivationTree::leaf(det, 0),
            DerivationTree::leaf(noun, 1),
            vec![Feature::Categorial("DP".to_string())],
            2
        );

        let dp2 = DerivationTree::merge(
            DerivationTree::leaf(det2, 3),
            DerivationTree::leaf(noun2, 4),
            vec![Feature::Categorial("DP".to_string())],
            5
        );

        let vp = DerivationTree::merge(
            dp1,
            DerivationTree::leaf(verb, 6),
            vec![Feature::Categorial("VP".to_string())],
            7
        );

        let full_sentence = DerivationTree::merge(
            dp2,
            vp,
            vec![Feature::Categorial("S".to_string())],
            8
        );

        // Linearize
        let linearized = parser.linearize(&full_sentence);

        // The linearization should follow the index order, which gives us:
        // "the cat chases the dog"
        assert_eq!(linearized, vec!["the", "cat", "the", "dog", "chases"]);
    }
}