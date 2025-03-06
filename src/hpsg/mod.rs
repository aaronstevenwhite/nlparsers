//! Head-Driven Phrase Structure Grammar (HPSG) implementation
//!
//! This module provides an implementation of HPSG, a constraint-based,
//! lexicalist approach to grammatical theory.
//!
//! The main components of HPSG are:
//! - Type hierarchy: an inheritance-based type system
//! - Feature structures: typed attribute-value matrices
//! - Principles: universal grammatical constraints
//! - Lexicon: rich lexical entries with feature structures
//! - Rules: limited number of schema for combining structures

pub mod feature_structure;
pub mod type_hierarchy;
pub mod sign;
pub mod rule;
pub mod principle;
pub mod parser;
pub mod entry;
pub mod lexicon;
pub mod registry;

pub use feature_structure::{FeatureStructure, FeatureType, TypedValue};
pub use type_hierarchy::{TypeHierarchy, Type};
pub use sign::{Sign, Category};
pub use rule::{Rule, RuleSchema};
pub use principle::{Principle, HeadFeaturePrinciple, ValencePrinciple};
pub use parser::{HPSGParser, ParserConfig};
pub use entry::LexicalEntry;
pub use lexicon::Lexicon;
pub use crate::common::registry::Registry;

use crate::common::Category as CategoryTrait;
use crate::common::FeatureStructure as CommonFeatureStructure;

/// HPSG Category implementation as a Category trait
impl CategoryTrait for Category {
    type Features = CommonFeatureStructure;
    
    fn features(&self) -> Option<&Self::Features> {
        Some(&self.common_features)
    }
    
    fn unify_with(&self, other: &Self) -> Option<Self> {
        // Complex unification that handles HPSG-specific requirements
        self.unify(other)
    }
    
    fn is_atomic(&self) -> bool {
        // Check if this is a basic category without complex features
        match &self.feature_structure {
            Some(fs) => fs.is_empty(),
            None => true
        }
    }
    
    fn atomic_name(&self) -> Option<&str> {
        // Only return name if truly atomic
        if self.is_atomic() {
            Some(&self.name)
        } else {
            None
        }
    }
}