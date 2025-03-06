//! Lexical entries for Lexical-Functional Grammar
//!
//! In LFG, lexical entries specify both C-structure categories and
//! F-structure constraints.

use std::fmt;
use crate::lfg::c_structure::Category;
use crate::lfg::f_structure::{FConstraint, FStructure, FValue};

/// A lexical entry in LFG
#[derive(Debug, Clone)]
pub struct LexicalEntry {
    /// The word form
    pub word: String,
    /// Syntactic category for C-structure
    pub category: Category,
    /// Semantic form (predicate-argument structure)
    pub pred: Option<(String, Vec<String>)>,
    /// Additional F-structure constraints
    pub constraints: Vec<FConstraint>,
}

impl LexicalEntry {
    /// Create a new lexical entry with just word and category
    pub fn new(word: &str, category: Category) -> Self {
        Self {
            word: word.to_string(),
            category,
            pred: None,
            constraints: Vec::new(),
        }
    }
    
    /// Create a lexical entry with predicate-argument structure
    pub fn with_pred(word: &str, category: Category, pred: &str, args: Vec<&str>) -> Self {
        let arguments: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        
        Self {
            word: word.to_string(),
            category,
            pred: Some((pred.to_string(), arguments)),
            constraints: Vec::new(),
        }
    }
    
    /// Add an F-structure constraint
    pub fn add_constraint(&mut self, constraint: FConstraint) {
        self.constraints.push(constraint);
    }
    
    /// Add multiple F-structure constraints
    pub fn add_constraints(&mut self, constraints: Vec<FConstraint>) {
        self.constraints.extend(constraints);
    }
    
    /// Create a lexical entry with constraints
    pub fn with_constraints(
        word: &str, 
        category: Category, 
        constraints: Vec<FConstraint>
    ) -> Self {
        Self {
            word: word.to_string(),
            category,
            pred: None,
            constraints,
        }
    }
    
    /// Create a complete lexical entry with predicate and constraints
    pub fn complete(
        word: &str, 
        category: Category, 
        pred: &str, 
        args: Vec<&str>,
        constraints: Vec<FConstraint>
    ) -> Self {
        let arguments: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        
        Self {
            word: word.to_string(),
            category,
            pred: Some((pred.to_string(), arguments)),
            constraints,
        }
    }
    
    /// Generate an F-structure for this lexical entry
    pub fn generate_f_structure(&self, id: usize) -> FStructure {
        let mut fs = FStructure::new(id);
        
        // Add predicate-argument structure if present
        if let Some((pred, args)) = &self.pred {
            fs.set_pred(pred, args.iter().map(|s| s.as_str()).collect());
        }
        
        // Apply constraints
        // In a real implementation, this would interpret the constraints
        // For now, we just add some simple attributes based on the constraints
        for constraint in &self.constraints {
            match constraint {
                FConstraint::Equation(lhs, rhs) => {
                    // Simple case: attribute = value
                    if !lhs.contains("↑") && !rhs.contains("↑") {
                        fs.set(lhs, FValue::Atomic(rhs.clone()));
                    }
                },
                FConstraint::ConstrainingEquation(lhs, rhs) => {
                    // Add constraining equations as regular values for now
                    if !lhs.contains("↑") && !rhs.contains("↑") {
                        fs.set(lhs, FValue::Atomic(rhs.clone()));
                    }
                },
                _ => {
                    // Other constraint types would need more complex processing
                }
            }
        }
        
        fs
    }
}

impl fmt::Display for LexicalEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.word, self.category)?;
        
        if let Some((pred, args)) = &self.pred {
            write!(f, " PRED='{}(", pred)?;
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", arg)?;
            }
            write!(f, ")'")?;
        }
        
        if !self.constraints.is_empty() {
            for constraint in &self.constraints {
                write!(f, " {}", constraint)?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lexical_entry_creation() {
        let category = Category::new("N");
        let entry = LexicalEntry::new("cat", category);
        
        assert_eq!(entry.word, "cat");
        assert_eq!(entry.category.name, "N");
        assert!(entry.pred.is_none());
        assert!(entry.constraints.is_empty());
    }
    
    #[test]
    fn test_entry_with_pred() {
        let category = Category::new("V");
        let entry = LexicalEntry::with_pred("sees", category, "see", vec!["SUBJ", "OBJ"]);
        
        assert_eq!(entry.word, "sees");
        assert_eq!(entry.category.name, "V");
        assert!(entry.pred.is_some());
        
        if let Some((pred, args)) = &entry.pred {
            assert_eq!(pred, "see");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], "SUBJ");
            assert_eq!(args[1], "OBJ");
        }
    }
    
    #[test]
    fn test_entry_with_constraints() {
        let category = Category::new("V");
        
        let constraints = vec![
            FConstraint::Equation("↑SUBJ NUM".to_string(), "sg".to_string()),
            FConstraint::Equation("↑SUBJ PERS".to_string(), "3".to_string()),
        ];
        
        let entry = LexicalEntry::with_constraints("sleeps", category, constraints);
        
        assert_eq!(entry.word, "sleeps");
        assert_eq!(entry.constraints.len(), 2);
    }
    
    #[test]
    fn test_generate_f_structure() {
        let category = Category::new("V");
        
        let constraints = vec![
            FConstraint::Equation("NUM".to_string(), "sg".to_string()),
            FConstraint::Equation("PERS".to_string(), "3".to_string()),
        ];
        
        let entry = LexicalEntry::complete(
            "sleeps", 
            category, 
            "sleep", 
            vec!["SUBJ"],
            constraints
        );
        
        let fs = entry.generate_f_structure(1);
        
        // Check that PRED was set correctly
        if let Some(FValue::Semantic(pred, args)) = fs.get("PRED") {
            assert_eq!(pred, "sleep");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], "SUBJ");
        } else {
            panic!("Expected PRED");
        }
        
        // Check that constraints were applied
        assert_eq!(fs.get("NUM"), Some(&FValue::Atomic("sg".to_string())));
        assert_eq!(fs.get("PERS"), Some(&FValue::Atomic("3".to_string())));
    }
    
    #[test]
    fn test_display() {
        let category = Category::new("V");
        
        let constraints = vec![
            FConstraint::Equation("↑SUBJ NUM".to_string(), "sg".to_string()),
            FConstraint::Equation("↑TENSE".to_string(), "pres".to_string()),
        ];
        
        let entry = LexicalEntry::complete(
            "sleeps", 
            category, 
            "sleep", 
            vec!["SUBJ"],
            constraints
        );
        
        let display = format!("{}", entry);
        assert!(display.contains("sleeps [V]"));
        assert!(display.contains("PRED='sleep(SUBJ)'"));
        assert!(display.contains("↑SUBJ NUM=sg"));
        assert!(display.contains("↑TENSE=pres"));
    }
}