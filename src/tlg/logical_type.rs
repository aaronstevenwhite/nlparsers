//! Logical types for Type-Logical Grammar

use std::fmt;
use std::hash::Hash;
use crate::common::FeatureStructure;
use crate::tlg::modality::Modality;

/// Types of structural properties for modalities in Type-Logical Grammar
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StructuralProperty {
    /// Associativity: (A • B) • C = A • (B • C)
    Associativity,
    /// Commutativity: A • B = B • A
    Commutativity,
    /// Weakening (resource can be discarded): A • B → A
    Weakening,
    /// Contraction (resource can be duplicated): A → A • A
    Contraction,
    /// Permutation: captures non-peripheral extraction
    Permutation,
}

/// Types of logical formula in Type-Logical Grammar
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicalType {
    /// Atomic types like s, np, n
    Atomic(String, FeatureStructure),
    /// Right implication A → B
    RightImplication(Box<LogicalType>, Box<LogicalType>, Option<Modality>),
    /// Left implication A ← B
    LeftImplication(Box<LogicalType>, Box<LogicalType>, Option<Modality>),
    /// Product type A ⊗ B
    Product(Box<LogicalType>, Box<LogicalType>, Option<Modality>),
    /// Modal type ◇A (diamond)
    Diamond(Box<LogicalType>, Option<Modality>),
    /// Modal type □A (box)
    Box(Box<LogicalType>, Option<Modality>),
    /// First-order quantifier ∀x.A
    Universal(String, Box<LogicalType>),
    /// First-order quantifier ∃x.A
    Existential(String, Box<LogicalType>),
    /// Discontinuous types for Displacement Calculus (↑ operator)
    UpArrow(Box<LogicalType>, Box<LogicalType>, usize),
    /// Discontinuous types for Displacement Calculus (↓ operator)
    DownArrow(Box<LogicalType>, Box<LogicalType>, usize),
}

impl fmt::Display for LogicalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalType::Atomic(s, features) => {
                write!(f, "{}", s)?;
                if !features.features.is_empty() {
                    write!(f, "{}", features)?;
                }
                Ok(())
            },
            LogicalType::RightImplication(a, b, modality) => {
                let mod_str = if let Some(m) = modality {
                    format!("{}", m)
                } else {
                    "".to_string()
                };
                
                if Self::is_complex(a) {
                    write!(f, "({}){} → {}", a, mod_str, b)
                } else {
                    write!(f, "{}{} → {}", a, mod_str, b)
                }
            },
            LogicalType::LeftImplication(a, b, modality) => {
                let mod_str = if let Some(m) = modality {
                    format!("{}", m)
                } else {
                    "".to_string()
                };
                
                if Self::is_complex(b) {
                    write!(f, "{} ←{} ({})", a, mod_str, b)
                } else {
                    write!(f, "{} ←{} {}", a, mod_str, b)
                }
            },
            LogicalType::Product(a, b, modality) => {
                let mod_str = if let Some(m) = modality {
                    format!("{}", m)
                } else {
                    "".to_string()
                };
                
                write!(f, "{} ⊗{} {}", a, mod_str, b)
            },
            LogicalType::Diamond(a, modality) => {
                let mod_str = if let Some(m) = modality {
                    format!("{}", m)
                } else {
                    "".to_string()
                };
                
                write!(f, "◇{}{}", mod_str, a)
            },
            LogicalType::Box(a, modality) => {
                let mod_str = if let Some(m) = modality {
                    format!("{}", m)
                } else {
                    "".to_string()
                };
                
                write!(f, "□{}{}", mod_str, a)
            },
            LogicalType::Universal(var, a) => write!(f, "∀{}.{}", var, a),
            LogicalType::Existential(var, a) => write!(f, "∃{}.{}", var, a),
            LogicalType::UpArrow(a, b, i) => write!(f, "{} ↑{} {}", a, i, b),
            LogicalType::DownArrow(a, b, i) => write!(f, "{} ↓{} {}", a, i, b),
        }
    }
}

impl LogicalType {
    /// Helper to determine if a type needs parentheses in display
    fn is_complex(t: &LogicalType) -> bool {
        !matches!(t, LogicalType::Atomic(_, _) | LogicalType::Diamond(_, _) | LogicalType::Box(_, _))
    }

    /// Helper to create atomic types
    pub fn atomic(name: &str) -> Self {
        LogicalType::Atomic(name.to_string(), FeatureStructure::new())
    }
    
    /// Helper to create atomic types with features
    pub fn atomic_with_features(name: &str, features: &FeatureStructure) -> Self {
        LogicalType::Atomic(name.to_string(), features.clone())
    }

    /// Convenience method for creating S type
    pub fn s() -> Self {
        Self::atomic("s")
    }

    /// Convenience method for creating NP type
    pub fn np() -> Self {
        Self::atomic("np")
    }

    /// Convenience method for creating N type
    pub fn n() -> Self {
        Self::atomic("n")
    }

    /// Helper to create right implication
    pub fn right_impl(left: LogicalType, right: LogicalType) -> Self {
        LogicalType::RightImplication(Box::new(left), Box::new(right), None)
    }
    
    /// Helper to create right implication with modality
    pub fn right_impl_with_modality(left: LogicalType, right: LogicalType, modality: Modality) -> Self {
        LogicalType::RightImplication(Box::new(left), Box::new(right), Some(modality))
    }

    /// Helper to create left implication
    pub fn left_impl(left: LogicalType, right: LogicalType) -> Self {
        LogicalType::LeftImplication(Box::new(left), Box::new(right), None)
    }
    
    /// Helper to create left implication with modality
    pub fn left_impl_with_modality(left: LogicalType, right: LogicalType, modality: Modality) -> Self {
        LogicalType::LeftImplication(Box::new(left), Box::new(right), Some(modality))
    }

    /// Helper to create product type
    pub fn product(left: LogicalType, right: LogicalType) -> Self {
        LogicalType::Product(Box::new(left), Box::new(right), None)
    }
    
    /// Helper to create product type with modality
    pub fn product_with_modality(left: LogicalType, right: LogicalType, modality: Modality) -> Self {
        LogicalType::Product(Box::new(left), Box::new(right), Some(modality))
    }

    /// Helper to create diamond modal type
    pub fn diamond(inner: LogicalType) -> Self {
        LogicalType::Diamond(Box::new(inner), None)
    }
    
    /// Helper to create diamond modal type with modality
    pub fn diamond_with_modality(inner: LogicalType, modality: Modality) -> Self {
        LogicalType::Diamond(Box::new(inner), Some(modality))
    }

    /// Helper to create box modal type
    pub fn boxed(inner: LogicalType) -> Self {
        LogicalType::Box(Box::new(inner), None)
    }
    
    /// Helper to create box modal type with modality
    pub fn boxed_with_modality(inner: LogicalType, modality: Modality) -> Self {
        LogicalType::Box(Box::new(inner), Some(modality))
    }
    
    /// Helper to create up arrow for Displacement Calculus
    pub fn up_arrow(left: LogicalType, right: LogicalType, index: usize) -> Self {
        LogicalType::UpArrow(Box::new(left), Box::new(right), index)
    }
    
    /// Helper to create down arrow for Displacement Calculus
    pub fn down_arrow(left: LogicalType, right: LogicalType, index: usize) -> Self {
        LogicalType::DownArrow(Box::new(left), Box::new(right), index)
    }
    
    /// Get feature structure if this is an atomic type
    pub fn get_features(&self) -> Option<&FeatureStructure> {
        match self {
            LogicalType::Atomic(_, features) => Some(features),
            _ => None,
        }
    }
    
    /// Unify this type with another if they are compatible
    pub fn unify(&self, other: &LogicalType) -> Option<LogicalType> {
        match (self, other) {
            (LogicalType::Atomic(s1, f1), LogicalType::Atomic(s2, f2)) => {
                if s1 != s2 {
                    return None;
                }
                
                if let Some(unified_features) = f1.unify(f2) {
                    Some(LogicalType::Atomic(s1.clone(), unified_features))
                } else {
                    None
                }
            },
            (LogicalType::RightImplication(a1, b1, m1), LogicalType::RightImplication(a2, b2, m2)) => {
                if m1 != m2 {
                    return None;
                }
                
                if let (Some(unified_a), Some(unified_b)) = (a1.unify(a2), b1.unify(b2)) {
                    Some(LogicalType::RightImplication(
                        Box::new(unified_a),
                        Box::new(unified_b),
                        m1.clone()
                    ))
                } else {
                    None
                }
            },
            (LogicalType::LeftImplication(a1, b1, m1), LogicalType::LeftImplication(a2, b2, m2)) => {
                if m1 != m2 {
                    return None;
                }
                
                if let (Some(unified_a), Some(unified_b)) = (a1.unify(a2), b1.unify(b2)) {
                    Some(LogicalType::LeftImplication(
                        Box::new(unified_a),
                        Box::new(unified_b),
                        m1.clone()
                    ))
                } else {
                    None
                }
            },
            (LogicalType::Product(a1, b1, m1), LogicalType::Product(a2, b2, m2)) => {
                if m1 != m2 {
                    return None;
                }
                
                if let (Some(unified_a), Some(unified_b)) = (a1.unify(a2), b1.unify(b2)) {
                    Some(LogicalType::Product(
                        Box::new(unified_a),
                        Box::new(unified_b),
                        m1.clone()
                    ))
                } else {
                    None
                }
            },
            (LogicalType::Diamond(a1, m1), LogicalType::Diamond(a2, m2)) => {
                if m1 != m2 {
                    return None;
                }
                
                if let Some(unified_a) = a1.unify(a2) {
                    Some(LogicalType::Diamond(Box::new(unified_a), m1.clone()))
                } else {
                    None
                }
            },
            (LogicalType::Box(a1, m1), LogicalType::Box(a2, m2)) => {
                if m1 != m2 {
                    return None;
                }
                
                if let Some(unified_a) = a1.unify(a2) {
                    Some(LogicalType::Box(Box::new(unified_a), m1.clone()))
                } else {
                    None
                }
            },
            (LogicalType::UpArrow(a1, b1, i1), LogicalType::UpArrow(a2, b2, i2)) => {
                if i1 != i2 {
                    return None;
                }
                
                if let (Some(unified_a), Some(unified_b)) = (a1.unify(a2), b1.unify(b2)) {
                    Some(LogicalType::UpArrow(
                        Box::new(unified_a),
                        Box::new(unified_b),
                        *i1
                    ))
                } else {
                    None
                }
            },
            (LogicalType::DownArrow(a1, b1, i1), LogicalType::DownArrow(a2, b2, i2)) => {
                if i1 != i2 {
                    return None;
                }
                
                if let (Some(unified_a), Some(unified_b)) = (a1.unify(a2), b1.unify(b2)) {
                    Some(LogicalType::DownArrow(
                        Box::new(unified_a),
                        Box::new(unified_b),
                        *i1
                    ))
                } else {
                    None
                }
            },
            _ => None, // Different type constructors don't unify
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{FeatureStructure, FeatureValue};

    #[test]
    fn test_logical_type_display() {
        let s = LogicalType::s();
        let np = LogicalType::np();
        
        let verb_type = LogicalType::left_impl(s.clone(), np.clone());
        assert_eq!(verb_type.to_string(), "s ← np");
        
        let tv_type = LogicalType::left_impl(verb_type.clone(), np.clone());
        assert_eq!(tv_type.to_string(), "(s ← np) ← np");
        
        let diamond_np = LogicalType::diamond(np.clone());
        assert_eq!(diamond_np.to_string(), "◇np");
    }
    
    #[test]
    fn test_with_features() {
        let mut sg_feat = FeatureStructure::new();
        sg_feat.add("num", FeatureValue::Atomic("sg".to_string()));
        let n_sg = LogicalType::atomic_with_features("n", &sg_feat);
        
        assert_eq!(n_sg.to_string(), "n[num=sg]");
    }
    
    #[test]
    fn test_unification() {
        let mut feat1 = FeatureStructure::new();
        feat1.add("num", FeatureValue::Atomic("sg".to_string()));
        
        let mut feat2 = FeatureStructure::new();
        feat2.add("per", FeatureValue::Atomic("3".to_string()));
        
        let cat1 = LogicalType::atomic_with_features("n", &feat1);
        let cat2 = LogicalType::atomic_with_features("n", &feat2);
        
        // Compatible features should unify
        let unified = cat1.unify(&cat2);
        assert!(unified.is_some());
        
        // Different types should not unify
        let cat3 = LogicalType::atomic_with_features("np", &feat1.clone());
        let unified2 = cat1.unify(&cat3);
        assert!(unified2.is_none());
    }
}