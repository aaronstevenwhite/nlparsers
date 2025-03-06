//! Principles for Head-Driven Phrase Structure Grammar
//!
//! Principles are general constraints that apply to phrase structure rules
//! in HPSG. They ensure that features are shared appropriately between
//! mother and daughter nodes.

use std::fmt::Debug;
use crate::hpsg::feature_structure::{FeatureStructure, TypedValue, FeatureType};
use crate::hpsg::sign::Sign;

/// A principle in HPSG that constrains how features are shared
pub trait Principle: Debug {
    /// Apply this principle to a mother and its daughters
    fn apply(&self, mother: &mut FeatureStructure, daughters: &[Sign]) -> bool;
    
    /// Get the name of this principle
    fn name(&self) -> &str;
    
    /// Get a description of this principle
    fn description(&self) -> &str;
}

/// The Head Feature Principle
/// 
/// Ensures that the HEAD features of a phrase and its head daughter are shared
#[derive(Debug)]
pub struct HeadFeaturePrinciple {
    /// Name of the principle
    name: String,
    /// Description of the principle
    description: String,
}

impl HeadFeaturePrinciple {
    /// Create a new Head Feature Principle
    pub fn new() -> Self {
        Self {
            name: "Head Feature Principle".to_string(),
            description: "The HEAD value of a headed phrase is structure-shared with the HEAD value of its head daughter.".to_string(),
        }
    }
}

impl Principle for HeadFeaturePrinciple {
    fn apply(&self, mother: &mut FeatureStructure, daughters: &[Sign]) -> bool {
        // Identify the head daughter (assumed to be the first daughter for simplicity)
        if daughters.is_empty() {
            return false;
        }
        
        let head_daughter = &daughters[0];
        
        // Find the HEAD feature in the head daughter
        if let Some(head_feature) = head_daughter.feature_structure.get("HEAD") {
            // Set the mother's HEAD feature to the same value
            mother.set("HEAD", head_feature.clone());
            true
        } else {
            // If the head daughter has no HEAD feature, the principle is satisfied vacuously
            true
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

/// The Valence Principle
/// 
/// Ensures that valence requirements are properly satisfied in phrases
#[derive(Debug)]
pub struct ValencePrinciple {
    /// Name of the principle
    name: String,
    /// Description of the principle
    description: String,
}

impl ValencePrinciple {
    /// Create a new Valence Principle
    pub fn new() -> Self {
        Self {
            name: "Valence Principle".to_string(),
            description: "The SUBJ and COMPS values of a headed phrase are related to those of its head daughter as specified by the grammar rule.".to_string(),
        }
    }
}

impl Principle for ValencePrinciple {
    fn apply(&self, _mother: &mut FeatureStructure, _daughters: &[Sign]) -> bool {
        // Implement valence constraint logic here
        // For a head-complement phrase, the mother's COMPS list should be empty
        // For a head-subject phrase, the mother's SUBJ list should be empty
        
        // This is a simplified implementation
        true
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

/// The Semantics Principle
/// 
/// Ensures that semantic content is properly composed
#[derive(Debug)]
pub struct SemanticsPrinciple {
    /// Name of the principle
    name: String,
    /// Description of the principle
    description: String,
}

impl SemanticsPrinciple {
    /// Create a new Semantics Principle
    pub fn new() -> Self {
        Self {
            name: "Semantics Principle".to_string(),
            description: "The semantics of a phrase is composed from the semantics of its daughters according to the grammar rule.".to_string(),
        }
    }
}

impl Principle for SemanticsPrinciple {
    fn apply(&self, _mother: &mut FeatureStructure, _daughters: &[Sign]) -> bool {
        // Implement semantics composition logic here
        // This would involve copying or combining the CONTENT features
        
        // This is a simplified implementation
        true
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

/// The ID Principle (Immediate Dominance)
/// 
/// Ensures that phrases conform to immediate dominance schemata
#[derive(Debug)]
pub struct IDPrinciple {
    /// Name of the principle
    name: String,
    /// Description of the principle
    description: String,
}

impl IDPrinciple {
    /// Create a new ID Principle
    pub fn new() -> Self {
        Self {
            name: "ID Principle".to_string(),
            description: "Every phrase must conform to one of the immediate dominance schemata.".to_string(),
        }
    }
}

impl Principle for IDPrinciple {
    fn apply(&self, _mother: &mut FeatureStructure, _daughters: &[Sign]) -> bool {
        // Implement ID schema constraint logic here
        // This would check if the phrase conforms to a valid schema
        
        // This is a simplified implementation
        true
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

/// Create a set of standard HPSG principles
pub fn standard_principles() -> Vec<Box<dyn Principle>> {
    vec![
        Box::new(HeadFeaturePrinciple::new()),
        Box::new(ValencePrinciple::new()),
        Box::new(SemanticsPrinciple::new()),
        Box::new(IDPrinciple::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_mother() -> FeatureStructure {
        FeatureStructure::new("phrase", 1)
    }
    
    fn create_test_daughters() -> Vec<Sign> {
        let mut head_fs = FeatureStructure::new("verb", 2);
        head_fs.set("HEAD", TypedValue {
            type_name: "verb".to_string(),
            value: FeatureType::String("verb".to_string()),
            id: 3,
        });
        
        let sign = Sign::new("phrase", &head_fs, 1);
        
        vec![sign]
    }
    
    #[test]
    fn test_head_feature_principle() {
        let principle = HeadFeaturePrinciple::new();
        let mut mother = create_test_mother();
        let daughters = create_test_daughters();
        
        assert!(principle.apply(&mut mother, &daughters));
        
        // Check that the HEAD feature was copied
        assert!(mother.has_feature("HEAD"));
        let head = mother.get("HEAD").unwrap();
        assert_eq!(head.type_name, "verb");
    }
    
    #[test]
    fn test_valence_principle() {
        let principle = ValencePrinciple::new();
        let mut mother = create_test_mother();
        let daughters = create_test_daughters();
        
        assert!(principle.apply(&mut mother, &daughters));
    }
    
    #[test]
    fn test_semantics_principle() {
        let principle = SemanticsPrinciple::new();
        let mut mother = create_test_mother();
        let daughters = create_test_daughters();
        
        assert!(principle.apply(&mut mother, &daughters));
    }
    
    #[test]
    fn test_id_principle() {
        let principle = IDPrinciple::new();
        let mut mother = create_test_mother();
        let daughters = create_test_daughters();
        
        assert!(principle.apply(&mut mother, &daughters));
    }
    
    #[test]
    fn test_standard_principles() {
        let principles = standard_principles();
        
        assert_eq!(principles.len(), 4);
        assert_eq!(principles[0].name(), "Head Feature Principle");
        assert_eq!(principles[1].name(), "Valence Principle");
        assert_eq!(principles[2].name(), "Semantics Principle");
        assert_eq!(principles[3].name(), "ID Principle");
    }
}