//! Grammar rules for Lexical-Functional Grammar
//!
//! LFG grammar rules are context-free rules augmented with functional constraints.

use std::fmt;
use crate::lfg::c_structure::Category;
use crate::lfg::f_structure::FConstraint;

/// A grammar rule in LFG
#[derive(Debug, Clone)]
pub struct Rule {
    /// Left-hand side category
    pub lhs: Category,
    /// Right-hand side categories
    pub rhs: Vec<Category>,
    /// Rule annotations (constraints)
    pub annotations: Vec<(usize, Vec<FConstraint>)>, // Index of RHS constituent and its constraints
    /// Name of the rule (for display and debugging)
    pub name: Option<String>,
}

impl Rule {
    /// Create a new rule with just categories
    pub fn new(lhs: Category, rhs: Vec<Category>) -> Self {
        Self {
            lhs,
            rhs,
            annotations: Vec::new(),
            name: None,
        }
    }
    
    /// Create a rule with a name
    pub fn with_name(lhs: Category, rhs: Vec<Category>, name: &str) -> Self {
        Self {
            lhs,
            rhs,
            annotations: Vec::new(),
            name: Some(name.to_string()),
        }
    }
    
    /// Add constraints to a constituent (0-based index)
    pub fn annotate(&mut self, index: usize, constraints: Vec<FConstraint>) {
        if index < self.rhs.len() {
            self.annotations.push((index, constraints));
        } else {
            panic!("Index out of range: {} >= {}", index, self.rhs.len());
        }
    }
    
    /// Add constraints directly to the right-hand side categories
    pub fn with_constraints(lhs: Category, rhs: Vec<(Category, Vec<FConstraint>)>) -> Self {
        let mut processed_rhs = Vec::new();
        let mut annotations = Vec::new();
        
        for (i, (cat, constraints)) in rhs.into_iter().enumerate() {
            if !constraints.is_empty() {
                annotations.push((i, constraints));
            }
            processed_rhs.push(cat);
        }
        
        Self {
            lhs,
            rhs: processed_rhs,
            annotations,
            name: None,
        }
    }
    
    /// Create a rule with common up-down equations
    pub fn with_up_annotations(lhs: Category, rhs: Vec<(Category, Option<&str>)>) -> Self {
        let mut processed_rhs = Vec::new();
        let mut annotations = Vec::new();
        
        for (i, (cat, maybe_func)) in rhs.into_iter().enumerate() {
            if let Some(func) = maybe_func {
                // Add "↑FUNC = ↓" annotation
                let constraint = FConstraint::Equation(
                    format!("↑{}", func), 
                    "↓".to_string()
                );
                annotations.push((i, vec![constraint]));
            }
            processed_rhs.push(cat);
        }
        
        Self {
            lhs,
            rhs: processed_rhs,
            annotations,
            name: None,
        }
    }
    
    /// Check if this rule can apply to a sequence of categories
    pub fn matches(&self, categories: &[Category]) -> bool {
        if self.rhs.len() != categories.len() {
            return false;
        }
        
        // Check if each RHS category matches the input
        for (i, cat) in self.rhs.iter().enumerate() {
            if cat.name != categories[i].name {
                return false;
            }
            
            // In a full implementation, we would check feature unification here
        }
        
        true
    }
    
    /// Create a common S -> NP VP rule
    pub fn s_rule() -> Self {
        let s = Category::s();
        let np = Category::np();
        let vp = Category::vp();
        
        let mut rule = Self::new(s, vec![np, vp]);
        rule.annotate(0, vec![FConstraint::Equation("↑SUBJ".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.name = Some("S -> NP VP".to_string());
        
        rule
    }
    
    /// Create a common VP -> V NP rule
    pub fn vp_rule() -> Self {
        let vp = Category::vp();
        let v = Category::new("V");
        let np = Category::np();
        
        let mut rule = Self::new(vp, vec![v, np]);
        rule.annotate(0, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![FConstraint::Equation("↑OBJ".to_string(), "↓".to_string())]);
        rule.name = Some("VP -> V NP".to_string());
        
        rule
    }
    
    /// Create a common NP -> Det N rule
    pub fn np_rule() -> Self {
        let np = Category::np();
        let det = Category::new("Det");
        let n = Category::new("N");
        
        let mut rule = Self::new(np, vec![det, n]);
        rule.annotate(0, vec![FConstraint::Equation("↑DET".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        rule.name = Some("NP -> Det N".to_string());
        
        rule
    }
    
    /// Get the expected length of a matching right-hand side
    pub fn rhs_length(&self) -> usize {
        self.rhs.len()
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the rule in traditional form
        write!(f, "{} ->", self.lhs)?;
        
        for (i, cat) in self.rhs.iter().enumerate() {
            write!(f, " {}", cat)?;
            
            // Find annotations for this constituent
            for (index, constraints) in &self.annotations {
                if *index == i {
                    write!(f, "[")?;
                    for (j, constraint) in constraints.iter().enumerate() {
                        if j > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", constraint)?;
                    }
                    write!(f, "]")?;
                    break;
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rule_creation() {
        let s = Category::s();
        let np = Category::np();
        let vp = Category::vp();
        
        let rule = Rule::new(s.clone(), vec![np.clone(), vp.clone()]);
        
        assert_eq!(rule.lhs.name, "S");
        assert_eq!(rule.rhs.len(), 2);
        assert_eq!(rule.rhs[0].name, "NP");
        assert_eq!(rule.rhs[1].name, "VP");
        assert!(rule.annotations.is_empty());
    }
    
    #[test]
    fn test_rule_annotation() {
        let s = Category::s();
        let np = Category::np();
        let vp = Category::vp();
        
        let mut rule = Rule::new(s.clone(), vec![np.clone(), vp.clone()]);
        
        // Add annotations
        rule.annotate(0, vec![FConstraint::Equation("↑SUBJ".to_string(), "↓".to_string())]);
        rule.annotate(1, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
        
        assert_eq!(rule.annotations.len(), 2);
        assert_eq!(rule.annotations[0].0, 0);
        assert_eq!(rule.annotations[1].0, 1);
    }
    
    #[test]
    fn test_rule_matching() {
        let s = Category::s();
        let np = Category::np();
        let vp = Category::vp();
        
        let rule = Rule::new(s.clone(), vec![np.clone(), vp.clone()]);
        
        // Matching categories
        assert!(rule.matches(&[np.clone(), vp.clone()]));
        
        // Wrong number of categories
        assert!(!rule.matches(&[np.clone()]));
        
        // Wrong categories
        let pp = Category::new("PP");
        assert!(!rule.matches(&[np.clone(), pp]));
    }
    
    #[test]
    fn test_rule_helpers() {
        // Test S rule
        let s_rule = Rule::s_rule();
        assert_eq!(s_rule.lhs.name, "S");
        assert_eq!(s_rule.rhs.len(), 2);
        assert_eq!(s_rule.annotations.len(), 2);
        
        // Test VP rule
        let vp_rule = Rule::vp_rule();
        assert_eq!(vp_rule.lhs.name, "VP");
        assert_eq!(vp_rule.rhs.len(), 2);
        assert_eq!(vp_rule.annotations.len(), 2);
        
        // Test NP rule
        let np_rule = Rule::np_rule();
        assert_eq!(np_rule.lhs.name, "NP");
        assert_eq!(np_rule.rhs.len(), 2);
        assert_eq!(np_rule.annotations.len(), 2);
    }
    
    #[test]
    fn test_rule_display() {
        let s_rule = Rule::s_rule();
        let display = format!("{}", s_rule);
        
        assert!(display.contains("S ->"));
        assert!(display.contains("NP[↑SUBJ=↓]"));
        assert!(display.contains("VP[↑=↓]"));
    }
    
    #[test]
    fn test_with_up_annotations() {
        let s = Category::s();
        let np = Category::np();
        let vp = Category::vp();
        
        let rule = Rule::with_up_annotations(
            s.clone(),
            vec![
                (np.clone(), Some("SUBJ")),
                (vp.clone(), None),
            ]
        );
        
        assert_eq!(rule.lhs.name, "S");
        assert_eq!(rule.rhs.len(), 2);
        assert_eq!(rule.annotations.len(), 1);
        assert_eq!(rule.annotations[0].0, 0);
        
        let constraint = &rule.annotations[0].1[0];
        match constraint {
            FConstraint::Equation(lhs, rhs) => {
                assert_eq!(lhs, "↑SUBJ");
                assert_eq!(rhs, "↓");
            },
            _ => panic!("Expected equation constraint")
        }
    }
}