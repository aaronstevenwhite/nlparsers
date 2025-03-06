//! # Rust Grammar Formalisms
//!
//! A comprehensive library for various grammatical formalisms in computational linguistics:
//! 
//! - **Combinatory Categorial Grammar (CCG)**: A lexicalized grammar formalism based on
//!   combinatory logic that elegantly handles coordination, extraction, and other phenomena.
//!
//! - **Minimalist Grammar (MG)**: A formalism based on Chomsky's Minimalist Program,
//!   with operations like Merge and Move, supporting displacement and phase-based derivation.
//!
//! - **Type-Logical Grammar (TLG)**: A grammar formalism based on logical deduction,
//!   with a strong connection to lambda calculus and formal semantics.
//!
//! - **Lexical-Functional Grammar (LFG)**: A non-transformational, constraint-based formalism
//!   with parallel c-structure (constituent) and f-structure (functional) representations.
//!
//! - **Head-Driven Phrase Structure Grammar (HPSG)**: A constraint-based, lexicalist approach
//!   to grammatical theory that uses typed feature structures and inheritance hierarchies.
//!
//! This library provides the necessary data structures and algorithms to work with these
//! formalisms, supporting features like morphosyntactic agreement, cross-linguistic
//! variation, and efficient parsing algorithms.

// Common modules used across formalisms
pub mod common;

// Specific grammar formalisms
#[cfg(feature = "ccg")]
pub mod ccg;

#[cfg(feature = "mg")]
pub mod mg;

#[cfg(feature = "hpsg")]
pub mod hpsg;

// Re-export commonly used items
#[cfg(feature = "ccg")]
pub use ccg::{CCGParser, CCGCategory, CCGNode};

#[cfg(feature = "mg")]
pub use mg::{MinimalistParser, Feature, LexicalItem as MGLexicalItem, DerivationTree};

#[cfg(feature = "hpsg")]
pub use hpsg::{HPSGParser, Rule, RuleSchema, Category, Sign, HeadFeaturePrinciple, ValencePrinciple, FeatureStructure};

pub use common::{FeatureValue, FeatureRegistry, Lexicon};