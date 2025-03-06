//! Lexical-Realizational Functional Grammar (LRFG) implementation
//!
//! This module provides an implementation of Lexical-Realizational Functional Grammar,
//! which extends Lexical-Functional Grammar with a realizational component.
//!
//! The main components of LRFG are:
//! - C-structure (constituent structure): represents phrase structure as trees (inherited from LFG)
//! - F-structure (functional structure): represents grammatical functions and relations (inherited from LFG)
//! - R-structure (realizational structure): mediates between f-structure and phonological form
//! - Lexical entries: mappings from words to c-structure categories with f-structure annotations
//! - Vocabulary items: mappings from r-structure features to phonological forms

pub mod r_structure;
pub mod mapping;
pub mod vocabulary;
pub mod parser;
pub mod registry;

// Re-export components from LFG
pub use crate::lfg::{
    CStructure, CNode, Category,
    FStructure, FValue, FConstraint,
    Rule, Constraint, Lexicon, AtomicCategoryRegistry as LFGAtomicCategoryRegistry
};

// Export LRFG-specific components
pub use r_structure::{RStructure, RFeature, RNode};
pub use mapping::{FRMapping, MappingRule};
pub use vocabulary::{VocabularyItem, Vocabulary};
pub use parser::{LRFGParser, ParserConfig};
pub use registry::{AtomicCategoryRegistry, AtomicCategory};
