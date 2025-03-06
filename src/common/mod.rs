//! Common data structures and functionality shared across grammar formalisms

pub mod feature;
pub mod lexicon;
pub mod registry;
pub mod error;

pub use feature::{FeatureValue, FeatureStructure, FeatureRegistry};
pub use lexicon::Lexicon;
pub use registry::AtomicTypeRegistry;
pub use error::Error;

/// Trait representing a grammatical category
/// 
/// This trait is implemented by different category types in various grammar formalisms,
/// providing a common interface for operations like unification.
pub trait Category: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash {
    /// The type of features used by this category
    type Features;
    
    /// Get the features associated with this category (if any)
    fn features(&self) -> Option<&Self::Features>;
    
    /// Attempt to unify this category with another compatible category
    fn unify_with(&self, other: &Self) -> Option<Self>;
    
    /// Check if this category is atomic (a primitive type)
    fn is_atomic(&self) -> bool;
    
    /// Get the name of this category if it's atomic
    fn atomic_name(&self) -> Option<&str>;
}

/// Trait representing a feature in a grammatical system
pub trait Feature: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash {
    /// Get the name of this feature
    fn name(&self) -> &str;
    
    /// Check if this feature matches another feature (for checking compatibility)
    fn is_matching(&self, other: &Self) -> bool;
}

/// Trait representing a node in a parse tree
pub trait ParseNode: std::fmt::Debug + Clone {
    /// The type of category used in this node
    type Cat: Category;
    
    /// Get the category of this node
    fn category(&self) -> &Self::Cat;
    
    /// Get the word associated with this node (if it's a leaf)
    fn word(&self) -> Option<&str>;
    
    /// Get the children of this node
    fn children(&self) -> &[Self];
    
    /// Get the rule used to create this node (if it's not a leaf)
    fn rule(&self) -> Option<&str>;
    
    /// Check if this node is a leaf
    fn is_leaf(&self) -> bool {
        self.children().is_empty()
    }
    
    /// Get any additional features or annotations specific to this node
    fn node_features(&self) -> Option<&FeatureStructure> {
        None
    }
}

/// Trait for grammatical parsers with common operations
pub trait Parser {
    /// The type of category used by this parser
    type Cat: Category;
    
    /// The type of parse node produced by this parser
    type Node: ParseNode<Cat = Self::Cat>;
    
    /// The type of configuration used by this parser
    type Config;
    
    /// Parse a sentence and return a parse tree if successful
    fn parse(&self, sentence: &str) -> Option<Self::Node>;
    
    /// Add a word with a category to the lexicon
    fn add_to_lexicon(&mut self, word: &str, category: Self::Cat);
    
    /// Get the configuration of this parser
    fn config(&self) -> &Self::Config;
    
    /// Set the configuration of this parser
    fn set_config(&mut self, config: Self::Config);
    
    /// Create a category with features
    fn create_category_with_features(&self, name: &str, features: &[(&str, &str)]) -> Result<Self::Cat, Error>;
    
    /// Get all possible parses for a sentence
    fn parse_all(&self, sentence: &str) -> Vec<Self::Node> {
        self.parse(sentence).into_iter().collect()
    }
}