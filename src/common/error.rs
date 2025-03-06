//! Error types for grammar formalisms

use thiserror::Error;

/// Error type for grammar formalism operations
#[derive(Error, Debug)]
pub enum Error {
    /// Unregistered type error
    #[error("Unregistered type: {0}")]
    UnregisteredType(String),
    
    /// Unregistered feature error
    #[error("Unregistered feature: {0}")]
    UnregisteredFeature(String),
    
    /// Invalid feature value error
    #[error("Invalid value '{value}' for feature '{feature}'")]
    InvalidFeatureValue {
        feature: String,
        value: String,
    },
    
    /// Lexicon error
    #[error("Lexicon error: {0}")]
    LexiconError(String),
    
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Feature unification error
    #[error("Feature unification error: {0}")]
    FeatureUnificationError(String),
    
    /// Category unification error
    #[error("Category unification error: {0}")]
    CategoryUnificationError(String),
    
    /// Invalid operation for grammar formalism
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

/// Result type for operations that can fail
pub type Result<T> = std::result::Result<T, Error>;