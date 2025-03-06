//! Type-Logical Grammar (TLG) implementation

pub mod logical_type;
pub mod parser;
pub mod proof;
pub mod modality;
pub mod proof_net;
pub mod registry;
pub mod lexicon;

pub use logical_type::{LogicalType, StructuralProperty};
pub use parser::{TLGParser, ParserConfig};
pub use proof::{ProofNode, ProofSearchState};
pub use modality::Modality;
pub use proof_net::ProofNet;
pub use lexicon::Lexicon;
pub use registry::AtomicTypeRegistry;

use crate::common::Category as CategoryTrait;

impl CategoryTrait for LogicalType {
    type Features = crate::common::FeatureStructure;
    
    fn features(&self) -> Option<&Self::Features> {
        match self {
            LogicalType::Atomic(_, features) => Some(features),
            _ => None,
        }
    }
    
    fn unify_with(&self, other: &Self) -> Option<Self> {
        self.unify(other)
    }
    
    fn is_atomic(&self) -> bool {
        matches!(self, LogicalType::Atomic(_, _))
    }
    
    fn atomic_name(&self) -> Option<&str> {
        match self {
            LogicalType::Atomic(name, _) => Some(name),
            _ => None,
        }
    }
}