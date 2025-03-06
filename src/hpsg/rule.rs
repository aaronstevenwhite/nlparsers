//! Grammar rules for Head-Driven Phrase Structure Grammar
//!
//! HPSG uses a small number of general schema for combining signs,
//! together with principles that constrain how these schema apply.

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use crate::hpsg::sign::Sign;
use crate::hpsg::principle::Principle;
use crate::hpsg::type_hierarchy::TypeHierarchy;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hpsg::feature_structure::{FeatureStructure, TypedValue, FeatureType};
    use crate::hpsg::principle::HeadFeaturePrinciple;
    
    fn create_test_signs() -> Vec<Sign> {
        let mut head_fs = FeatureStructure::new("verb", 1);
        head_fs.set("HEAD", TypedValue {
            type_name: "verb".to_string(),
            value: FeatureType::String("verb".to_string()),
            id: 2,
        });
        head_fs.set("PHON", TypedValue {
            type_name: "string".to_string(),
            value: FeatureType::String("see".to_string()),
            id: 3,
        });
        
        let mut comp_fs = FeatureStructure::new("noun", 4);
        comp_fs.set("HEAD", TypedValue {
            type_name: "noun".to_string(),
            value: FeatureType::String("noun".to_string()),
            id: 5,
        });
        comp_fs.set("PHON", TypedValue {
            type_name: "string".to_string(),
            value: FeatureType::String("cat".to_string()),
            id: 6,
        });
        
        let head = Sign::lexical("see", head_fs, 1);
        let complement = Sign::lexical("cat", comp_fs, 2);
        
        vec![head, complement]
    }
    
    #[test]
    fn test_rule_creation() {
        let rule = Rule::new("head-comp", RuleSchema::HeadComplement);
        
        assert_eq!(rule.name, "head-comp");
        assert_eq!(rule.schema, RuleSchema::HeadComplement);
        assert!(rule.mother_constraints.is_empty());
        assert!(rule.daughter_constraints.is_empty());
        assert!(rule.principles.is_empty());
    }
    
    #[test]
    fn test_rule_constraints() {
        let mut rule = Rule::new("head-comp", RuleSchema::HeadComplement);
        
        rule.add_mother_constraint("SYNSEM.LOCAL.CAT.HEAD = #1");
        rule.add_daughter_constraint("HEAD-DTR.SYNSEM.LOCAL.CAT.HEAD = #1");
        
        assert_eq!(rule.mother_constraints.len(), 1);
        assert_eq!(rule.daughter_constraints.len(), 1);
    }
    
    #[test]
    fn test_rule_principles() {
        let principle = Rc::new(HeadFeaturePrinciple::new()) as Rc<dyn Principle>;
        
        let mut rule = Rule::new("head-comp", RuleSchema::HeadComplement);
        rule.add_principle(principle);
        
        assert_eq!(rule.principles.len(), 1);
    }
    
    #[test]
    fn test_rule_application() {
        let principle = Rc::new(HeadFeaturePrinciple::new()) as Rc<dyn Principle>;
        
        let mut rule = Rule::new("head-comp", RuleSchema::HeadComplement);
        rule.add_principle(principle);
        
        let daughters = create_test_signs();
        
        let result = rule.apply(&daughters, 3);
        assert!(result.is_some());
        
        let mother = result.unwrap();
        assert_eq!(mother.sign_type, "phrase");
        assert_eq!(mother.daughters.len(), 2);
        assert_eq!(mother.index, 3);
    }
    
    #[test]
    fn test_rule_schema_display() {
        assert_eq!(format!("{}", RuleSchema::HeadComplement), "Head-Complement");
        assert_eq!(format!("{}", RuleSchema::HeadSubject), "Head-Subject");
        assert_eq!(format!("{}", RuleSchema::HeadAdjunct), "Head-Adjunct");
        assert_eq!(
            format!("{}", RuleSchema::Custom("my-rule".to_string())), 
            "my-rule"
        );
    }
    
    #[test]
    fn test_rule_display() {
        let mut rule = Rule::new("head-comp", RuleSchema::HeadComplement);
        rule.add_mother_constraint("SYNSEM.LOCAL.CAT.HEAD = #1");
        
        let display = format!("{}", rule);
        assert!(display.contains("head-comp"));
        assert!(display.contains("Head-Complement"));
        assert!(display.contains("Mother constraints"));
    }
}

/// Types of schema in HPSG
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleSchema {
    /// Head-complement schema (head combines with its complements)
    HeadComplement,
    /// Head-subject schema (head combines with its subject)
    HeadSubject,
    /// Head-adjunct schema (head combines with a modifier)
    HeadAdjunct,
    /// Head-filler schema (for displaced constituents like in wh-questions)
    HeadFiller,
    /// Specifier-head schema (for determiners, etc.)
    SpecifierHead,
    /// Coordination schema (for coordinated phrases)
    Coordination,
    /// Custom schema with a name
    Custom(String),
}

impl fmt::Display for RuleSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleSchema::HeadComplement => write!(f, "Head-Complement"),
            RuleSchema::HeadSubject => write!(f, "Head-Subject"),
            RuleSchema::HeadAdjunct => write!(f, "Head-Adjunct"),
            RuleSchema::HeadFiller => write!(f, "Head-Filler"),
            RuleSchema::SpecifierHead => write!(f, "Specifier-Head"),
            RuleSchema::Coordination => write!(f, "Coordination"),
            RuleSchema::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// A grammar rule in HPSG
#[derive(Debug, Clone)]
pub struct Rule {
    /// Name of the rule
    pub name: String,
    /// The schema this rule implements
    pub schema: RuleSchema,
    /// Constraints on the mother node
    pub mother_constraints: Vec<String>,
    /// Constraints on the daughters
    pub daughter_constraints: Vec<String>,
    /// Principles that this rule must satisfy
    pub principles: Vec<Rc<dyn Principle>>,
    /// Reference to the type hierarchy
    pub type_hierarchy: Option<Rc<RefCell<TypeHierarchy>>>,
}

impl Rule {
    /// Create a new rule with a name and schema
    pub fn new(name: &str, schema: RuleSchema) -> Self {
        Self {
            name: name.to_string(),
            schema,
            mother_constraints: Vec::new(),
            daughter_constraints: Vec::new(),
            principles: Vec::new(),
            type_hierarchy: None,
        }
    }
    
    /// Add a constraint on the mother node
    pub fn add_mother_constraint(&mut self, constraint: &str) {
        self.mother_constraints.push(constraint.to_string());
    }
    
    /// Add a constraint on daughters
    pub fn add_daughter_constraint(&mut self, constraint: &str) {
        self.daughter_constraints.push(constraint.to_string());
    }
    
    /// Add a principle that must be satisfied
    pub fn add_principle(&mut self, principle: Rc<dyn Principle>) {
        self.principles.push(principle);
    }
    
    /// Set the type hierarchy
    pub fn with_type_hierarchy(mut self, hierarchy: Rc<RefCell<TypeHierarchy>>) -> Self {
        self.type_hierarchy = Some(hierarchy);
        self
    }
    
    /// Try to apply this rule to a set of daughter signs
    pub fn apply(&self, daughters: &[Sign], next_index: usize) -> Option<Sign> {
        match self.schema {
            RuleSchema::HeadComplement => self.apply_head_complement(daughters, next_index),
            RuleSchema::HeadSubject => self.apply_head_subject(daughters, next_index),
            RuleSchema::HeadAdjunct => self.apply_head_adjunct(daughters, next_index),
            RuleSchema::SpecifierHead => self.apply_specifier_head(daughters, next_index),
            RuleSchema::HeadFiller => self.apply_head_filler(daughters, next_index),
            RuleSchema::Coordination => self.apply_coordination(daughters, next_index),
            RuleSchema::Custom(_) => self.apply_custom(daughters, next_index),
        }
    }
    
    /// Apply the Head-Complement schema
    fn apply_head_complement(&self, daughters: &[Sign], next_index: usize) -> Option<Sign> {
        // Need at least two daughters: head and complement(s)
        if daughters.len() < 2 {
            return None;
        }
        
        // Assume first daughter is the head in this implementation
        let head = &daughters[0];
        let _complements = &daughters[1..];
        
        // Create a feature structure for the mother
        let mut mother_fs = head.feature_structure.clone();
        
        // Apply principles
        for principle in &self.principles {
            if !principle.apply(&mut mother_fs, daughters) {
                return None;
            }
        }
        
        // Create the mother sign
        let sign = Sign::with_daughters(
            "phrase",
            &mother_fs,
            daughters.to_vec(),
            next_index
        );
        Some(sign)
    }
    
    /// Apply the Head-Subject schema
    fn apply_head_subject(&self, daughters: &[Sign], next_index: usize) -> Option<Sign> {
        // Need exactly two daughters: subject and head
        if daughters.len() != 2 {
            return None;
        }
        
        // Assume second daughter is the head in this implementation
        let _subject = &daughters[0];
        let head = &daughters[1];
        
        // Create a feature structure for the mother
        let mut mother_fs = head.feature_structure.clone();
        
        // Apply principles
        for principle in &self.principles {
            if !principle.apply(&mut mother_fs, daughters) {
                return None;
            }
        }
        
        // Create the mother sign
        let sign = Sign::with_daughters(
            "phrase",
            &mother_fs,
            daughters.to_vec(),
            next_index
        );
        Some(sign)
    }
    
    /// Apply the Head-Adjunct schema
    fn apply_head_adjunct(&self, daughters: &[Sign], next_index: usize) -> Option<Sign> {
        // Need exactly two daughters: head and adjunct
        if daughters.len() != 2 {
            return None;
        }
        
        // Assume first daughter is the head in this implementation
        let head = &daughters[0];
        let _adjunct = &daughters[1];
        
        // Create a feature structure for the mother
        let mut mother_fs = head.feature_structure.clone();
        
        // Apply principles
        for principle in &self.principles {
            if !principle.apply(&mut mother_fs, daughters) {
                return None;
            }
        }
        
        // Create the mother sign
        let sign = Sign::with_daughters(
            "phrase",
            &mother_fs,
            daughters.to_vec(),
            next_index
        );
        Some(sign)
    }
    
    /// Apply the Specifier-Head schema
    fn apply_specifier_head(&self, daughters: &[Sign], next_index: usize) -> Option<Sign> {
        // Need exactly two daughters: specifier and head
        if daughters.len() != 2 {
            return None;
        }
        
        // Assume second daughter is the head in this implementation
        let _specifier = &daughters[0];
        let head = &daughters[1];
        
        // Create a feature structure for the mother
        let mut mother_fs = head.feature_structure.clone();
        
        // Apply principles
        for principle in &self.principles {
            if !principle.apply(&mut mother_fs, daughters) {
                return None;
            }
        }
        
        // Create the mother sign
        let sign = Sign::with_daughters(
            "phrase",
            &mother_fs,
            daughters.to_vec(),
            next_index
        );
        Some(sign)
    }
    
    /// Apply the Head-Filler schema
    fn apply_head_filler(&self, daughters: &[Sign], next_index: usize) -> Option<Sign> {
        // Need exactly two daughters: filler and head
        if daughters.len() != 2 {
            return None;
        }
        
        // Assume second daughter is the head in this implementation
        let _filler = &daughters[0];
        let head = &daughters[1];
        
        // Create a feature structure for the mother
        let mut mother_fs = head.feature_structure.clone();
        
        // Apply principles
        for principle in &self.principles {
            if !principle.apply(&mut mother_fs, daughters) {
                return None;
            }
        }
        
        // Create the mother sign
        let sign = Sign::with_daughters(
            "phrase",
            &mother_fs,
            daughters.to_vec(),
            next_index
        );
        Some(sign)
    }
    
    /// Apply the Coordination schema
    fn apply_coordination(&self, daughters: &[Sign], next_index: usize) -> Option<Sign> {
        // Need at least three daughters: conjuncts and conjunction
        if daughters.len() < 3 {
            return None;
        }
        
        // In a real implementation, we would identify the conjuncts and conjunction
        // For now, just create a mother with all daughters
        
        // Create a feature structure for the mother
        // For simplicity, copy the first daughter's feature structure
        let mut mother_fs = daughters[0].feature_structure.clone();
        
        // Apply principles
        for principle in &self.principles {
            if !principle.apply(&mut mother_fs, daughters) {
                return None;
            }
        }
        
        // Create the mother sign
        let sign = Sign::with_daughters(
            "phrase",
            &mother_fs,
            daughters.to_vec(),
            next_index
        );
        Some(sign)
    }
    
    /// Apply a custom schema
    fn apply_custom(&self, daughters: &[Sign], next_index: usize) -> Option<Sign> {
        // This would implement custom rule logic
        // For now, just create a mother with all daughters
        
        // Create a feature structure for the mother
        // For simplicity, copy the first daughter's feature structure
        let mut mother_fs = daughters[0].feature_structure.clone();
        
        // Apply principles
        for principle in &self.principles {
            if !principle.apply(&mut mother_fs, daughters) {
                return None;
            }
        }
        
        // Create the mother sign
        let sign = Sign::with_daughters(
            "phrase",
            &mother_fs,
            daughters.to_vec(),
            next_index
        );
        Some(sign)
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.schema)?;
        
        if !self.mother_constraints.is_empty() {
            write!(f, "\nMother constraints:")?;
            for constraint in &self.mother_constraints {
                write!(f, "\n  {}", constraint)?;
            }
        }
        
        if !self.daughter_constraints.is_empty() {
            write!(f, "\nDaughter constraints:")?;
            for constraint in &self.daughter_constraints {
                write!(f, "\n  {}", constraint)?;
            }
        }
        
        Ok(())
    }
}