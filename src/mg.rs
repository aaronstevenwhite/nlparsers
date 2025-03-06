//! Minimalist Grammar (MG) implementation

pub mod feature;
pub mod lexical_item;
pub mod derivation;
pub mod parser;
pub mod workspace;
pub mod phase;

pub use feature::Feature;
pub use lexical_item::LexicalItem;
pub use derivation::DerivationTree;
pub use parser::{MinimalistParser, ParserConfig};
pub use crate::common::Parser;

use crate::common::Feature as FeatureTrait;

// Implementation of the Feature trait for MG features
impl FeatureTrait for Feature {
    fn name(&self) -> &str {
        match self {
            Feature::Categorial(s) => s,
            Feature::Selector(s) => s,
            Feature::Licensor(s) => s,
            Feature::Licensee(s) => s,
            Feature::StrongSelector(s) => s,
            Feature::AdjunctSelector(s) => s,
            Feature::Agreement(s, _) => s,
            Feature::Phase(s) => s,
            Feature::Delayed(f) => f.name(),
        }
    }
    
    fn is_matching(&self, other: &Self) -> bool {
        // Combine both matching behaviors from the Feature implementation
        self.matches(other) || self.matches_move(other)
    }
}