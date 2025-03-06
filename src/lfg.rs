//! Lexical-Functional Grammar (LFG) implementation
//!
//! This module provides an implementation of Lexical-Functional Grammar,
//! a non-transformational, constraint-based syntactic framework.
//!
//! The main components of LFG are:
//! - C-structure (constituent structure): represents phrase structure as trees
//! - F-structure (functional structure): represents grammatical functions and relations
//! - Lexical entries: mappings from words to c-structure categories with f-structure annotations

pub mod c_structure;
pub mod f_structure;
pub mod parser;
pub mod entry;
pub mod rule;
pub mod constraint;
pub mod registry;
pub mod lexicon;

pub use c_structure::{CStructure, CNode, Category};
pub use f_structure::{FStructure, FValue, FConstraint};
pub use parser::{LFGParser, ParserConfig};
pub use entry::LexicalEntry;
pub use rule::Rule;
pub use constraint::Constraint;
pub use lexicon::Lexicon;
pub use registry::AtomicCategoryRegistry;

use crate::common::Category as CategoryTrait;

impl CategoryTrait for Category {
    type Features = crate::common::FeatureStructure;
    
    fn features(&self) -> Option<&Self::Features> {
        Some(&self.features)
    }
    
    fn unify_with(&self, other: &Self) -> Option<Self> {
        self.unify(other)
    }
    
    fn is_atomic(&self) -> bool {
        true
    }
    
    fn atomic_name(&self) -> Option<&str> {
        Some(&self.name)
    }
}