//! Modalities for Type-Logical Grammar
//! 
//! This module defines the modality structures used in multi-modal
//! Type-Logical Grammar, which allow for controlled relaxation of
//! structural rules.

use std::fmt;
use std::collections::HashSet;
use crate::tlg::logical_type::StructuralProperty;
use std::hash::{Hash, Hasher};

/// A modality in Multi-Modal Type-Logical Grammar
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modality {
    /// The index of the modality, typically written as a subscript
    pub index: usize,
    /// The structural properties this modality has
    pub properties: HashSet<StructuralProperty>,
}

/// Manual implementation of Hash for Modality because HashSet doesn't implement Hash.
/// 
/// We can't use #[derive(Hash)] because HashSet<T> doesn't implement Hash, even when T does.
/// This is because HashSet is an unordered collection, and naively hashing its elements in
/// their internal storage order would produce different hash values for sets with the same
/// elements but different insertion orders.
/// 
/// This implementation sorts the properties before hashing them to ensure that two Modality
/// instances with the same index and the same set of properties will always hash to the same
/// value, regardless of the order in which those properties were added to the set.
impl Hash for Modality {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        // Sort the properties to ensure consistent hashing
        let mut properties: Vec<&StructuralProperty> = self.properties.iter().collect();
        // Use a stable sorting method based on the Debug representation
        properties.sort_by(|a, b| {
            format!("{:?}", a).cmp(&format!("{:?}", b))
        });
        for prop in properties {
            prop.hash(state);
        }
    }
}

impl Modality {
    /// Create a new modality with the given index
    pub fn new(index: usize) -> Self {
        Self {
            index,
            properties: HashSet::new(),
        }
    }
    
    /// Create a new modality with properties
    pub fn with_properties(index: usize, properties: Vec<StructuralProperty>) -> Self {
        Self {
            index,
            properties: properties.into_iter().collect(),
        }
    }
    
    /// Check if this modality has a particular structural property
    pub fn has_property(&self, property: &StructuralProperty) -> bool {
        self.properties.contains(property)
    }
    
    /// Add a structural property to this modality
    pub fn add_property(&mut self, property: StructuralProperty) {
        self.properties.insert(property);
    }
    
    /// Remove a structural property from this modality
    pub fn remove_property(&mut self, property: &StructuralProperty) {
        self.properties.remove(property);
    }
    
    /// Check if this modality allows associativity
    pub fn is_associative(&self) -> bool {
        self.has_property(&StructuralProperty::Associativity)
    }
    
    /// Check if this modality allows commutativity
    pub fn is_commutative(&self) -> bool {
        self.has_property(&StructuralProperty::Commutativity)
    }
    
    /// Check if this modality allows weakening
    pub fn allows_weakening(&self) -> bool {
        self.has_property(&StructuralProperty::Weakening)
    }
    
    /// Check if this modality allows contraction
    pub fn allows_contraction(&self) -> bool {
        self.has_property(&StructuralProperty::Contraction)
    }
    
    /// Check if this modality allows permutation
    pub fn allows_permutation(&self) -> bool {
        self.has_property(&StructuralProperty::Permutation)
    }
}

impl fmt::Display for Modality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_modality_creation() {
        let m1 = Modality::new(1);
        assert_eq!(m1.index, 1);
        assert!(m1.properties.is_empty());
        
        let m2 = Modality::with_properties(2, vec![
            StructuralProperty::Associativity,
            StructuralProperty::Commutativity,
        ]);
        
        assert_eq!(m2.index, 2);
        assert_eq!(m2.properties.len(), 2);
        assert!(m2.is_associative());
        assert!(m2.is_commutative());
        assert!(!m2.allows_weakening());
    }
    
    #[test]
    fn test_property_manipulation() {
        let mut m = Modality::new(3);
        
        m.add_property(StructuralProperty::Associativity);
        assert!(m.is_associative());
        
        m.add_property(StructuralProperty::Permutation);
        assert!(m.allows_permutation());
        
        m.remove_property(&StructuralProperty::Associativity);
        assert!(!m.is_associative());
        assert!(m.allows_permutation());
    }
    
    #[test]
    fn test_display() {
        let m = Modality::new(42);
        assert_eq!(m.to_string(), "42");
    }
}