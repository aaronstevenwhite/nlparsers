//! Combinatory Categorial Grammar (CCG) implementation

pub mod category;
pub mod parser;
pub mod rules;
pub mod node;

pub use category::CCGCategory;
pub use parser::{CCGParser, CCGParserConfig};
pub use node::CCGNode;

use crate::common::Category as CategoryTrait;

/// CCG Category implementation as a Category trait
impl CategoryTrait for CCGCategory {
    type Features = crate::common::FeatureStructure;
    
    fn features(&self) -> Option<&Self::Features> {
        match self {
            CCGCategory::Atomic(_, features) => Some(features),
            _ => None,
        }
    }
    
    fn unify_with(&self, other: &Self) -> Option<Self> {
        self.unify(other)
    }
    
    fn is_atomic(&self) -> bool {
        matches!(self, CCGCategory::Atomic(_, _))
    }
    
    fn atomic_name(&self) -> Option<&str> {
        match self {
            CCGCategory::Atomic(name, _) => Some(name),
            _ => None,
        }
    }
}