//! Functional structure (F-structure) for Lexical-Functional Grammar
//!
//! F-structure in LFG represents grammatical functions and relations.
//! It consists of attribute-value matrices with constraints.

use std::fmt;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// LFG functional constraints notation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FConstraint {
    /// Equality constraints (↑=↓, ↑SUBJ=↓, etc.)
    Equation(String, String),
    /// Containment constraints (↑∊↓, ↑ADJUNCTS∊↓, etc.)
    Containment(String, String),
    /// Negated existential constraints (¬(↑TENSE), etc.)
    Negation(String),
    /// Disjunctive constraints ((↑NUM)=c sg ∨ (↑NUM)=c pl)
    Disjunction(Box<FConstraint>, Box<FConstraint>),
    /// Defining equation (↑F ↓)
    DefiningEquation(String, String),
    /// Constraining equation (↑F =c ↓)
    ConstrainingEquation(String, String),
    /// Functional uncertainty (↑COMP* OBJ)=↓
    FunctionalUncertainty(String, String),
    /// Off-path constraint (↑ SUBJ NUM) =c (↑ OBJ NUM)
    OffPathConstraint(String, String),
    /// Inside-out functional uncertainty (ADJ ∈ (COMP* ↑))
    InsideOut(String, String),
    /// Set membership constraint (↑ ADJUNCTS ∋ ↓)
    SetMembership(String, String),
}

impl fmt::Display for FConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FConstraint::Equation(lhs, rhs) => write!(f, "{}={}", lhs, rhs),
            FConstraint::Containment(lhs, rhs) => write!(f, "{}∊{}", lhs, rhs),
            FConstraint::Negation(path) => write!(f, "¬({})", path),
            FConstraint::Disjunction(left, right) => write!(f, "({} ∨ {})", left, right),
            FConstraint::DefiningEquation(lhs, rhs) => write!(f, "{} {}", lhs, rhs),
            FConstraint::ConstrainingEquation(lhs, rhs) => write!(f, "{} =c {}", lhs, rhs),
            FConstraint::FunctionalUncertainty(lhs, rhs) => write!(f, "{}={}", lhs, rhs),
            FConstraint::OffPathConstraint(lhs, rhs) => write!(f, "{} =c {}", lhs, rhs),
            FConstraint::InsideOut(lhs, rhs) => write!(f, "{} ∈ {}", lhs, rhs),
            FConstraint::SetMembership(lhs, rhs) => write!(f, "{} ∋ {}", lhs, rhs),
        }
    }
}

/// Value types in F-structure
#[derive(Debug, Clone)]
pub enum FValue {
    /// Atomic values like strings, booleans, etc.
    Atomic(String),
    /// Embedded F-structures (recursive)
    Structure(Box<FStructure>),
    /// Set of values (for adjuncts, etc.)
    Set(Vec<FValue>),
    /// Semantic forms (PRED values)
    Semantic(String, Vec<String>),
}

impl PartialEq for FValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FValue::Atomic(s1), FValue::Atomic(s2)) => s1 == s2,
            (FValue::Structure(fs1), FValue::Structure(fs2)) => fs1 == fs2,
            (FValue::Set(set1), FValue::Set(set2)) => {
                if set1.len() != set2.len() {
                    return false;
                }
                
                // Compare sets (order-independent)
                let mut matched = vec![false; set2.len()];
                
                for val1 in set1 {
                    let mut found = false;
                    
                    for (i, val2) in set2.iter().enumerate() {
                        if !matched[i] && val1 == val2 {
                            matched[i] = true;
                            found = true;
                            break;
                        }
                    }
                    
                    if !found {
                        return false;
                    }
                }
                
                true
            },
            (FValue::Semantic(pred1, args1), FValue::Semantic(pred2, args2)) => {
                pred1 == pred2 && args1 == args2
            },
            _ => false,
        }
    }
}

impl Eq for FValue {}

impl Hash for FValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            FValue::Atomic(s) => {
                0u8.hash(state);
                s.hash(state);
            },
            FValue::Structure(fs) => {
                1u8.hash(state);
                fs.hash(state);
            },
            FValue::Set(set) => {
                2u8.hash(state);
                // Hash the set members (unordered)
                let mut hashes = Vec::new();
                for val in set {
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    val.hash(&mut hasher);
                    hashes.push(hasher.finish());
                }
                hashes.sort();
                hashes.hash(state);
            },
            FValue::Semantic(pred, args) => {
                3u8.hash(state);
                pred.hash(state);
                args.hash(state);
            },
        }
    }
}

impl fmt::Display for FValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FValue::Atomic(s) => write!(f, "{}", s),
            FValue::Structure(fs) => write!(f, "{}", fs),
            FValue::Set(set) => {
                write!(f, "{{ ")?;
                for (i, val) in set.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, " }}")
            },
            FValue::Semantic(pred, args) => {
                write!(f, "'{}(", pred)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")'")
            },
        }
    }
}

/// F-structure attribute-value matrix
#[derive(Debug, Clone)]
pub struct FStructure {
    /// Attribute-value pairs
    pub attributes: HashMap<String, FValue>,
    /// Unique identifier for this F-structure
    pub id: usize,
}

impl PartialEq for FStructure {
    fn eq(&self, other: &Self) -> bool {
        self.attributes == other.attributes
    }
}

impl Eq for FStructure {}

impl Hash for FStructure {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash based on attributes, not ID
        let mut keys: Vec<_> = self.attributes.keys().collect();
        keys.sort();
        
        for key in keys {
            key.hash(state);
            self.attributes.get(key).unwrap().hash(state);
        }
    }
}

impl FStructure {
    /// Create a new empty F-structure with an ID
    pub fn new(id: usize) -> Self {
        Self {
            attributes: HashMap::new(),
            id,
        }
    }
    
    /// Add an attribute-value pair
    pub fn set(&mut self, attribute: &str, value: FValue) {
        self.attributes.insert(attribute.to_string(), value);
    }
    
    /// Get a value by attribute name
    pub fn get(&self, attribute: &str) -> Option<&FValue> {
        self.attributes.get(attribute)
    }
    
    /// Get a mutable reference to a value
    pub fn get_mut(&mut self, attribute: &str) -> Option<&mut FValue> {
        self.attributes.get_mut(attribute)
    }
    
    /// Check if an attribute exists
    pub fn has_attribute(&self, attribute: &str) -> bool {
        self.attributes.contains_key(attribute)
    }
    
    /// Add or update a semantic form (PRED value)
    pub fn set_pred(&mut self, predicate: &str, arguments: Vec<&str>) {
        let args: Vec<String> = arguments.iter().map(|s| s.to_string()).collect();
        self.set("PRED", FValue::Semantic(predicate.to_string(), args));
    }
    
    /// Add a value to a set attribute
    pub fn add_to_set(&mut self, attribute: &str, value: FValue) {
        match self.get_mut(attribute) {
            Some(FValue::Set(set)) => {
                set.push(value);
            },
            Some(_) => {
                // Convert existing value to a set
                let existing = self.attributes.remove(attribute).unwrap();
                let set = vec![existing, value];
                self.set(attribute, FValue::Set(set));
            },
            None => {
                // Create a new set with this value
                self.set(attribute, FValue::Set(vec![value]));
            }
        }
    }
    
    /// Unify this F-structure with another
    pub fn unify(&self, other: &FStructure) -> Option<FStructure> {
        let mut result = self.clone();
        
        // For each attribute in the other F-structure
        for (attr, other_val) in &other.attributes {
            if let Some(self_val) = self.get(attr) {
                // Both have this attribute, unify the values
                match (self_val, other_val) {
                    (FValue::Atomic(s1), FValue::Atomic(s2)) => {
                        if s1 != s2 {
                            return None; // Unification failure
                        }
                    },
                    (FValue::Structure(fs1), FValue::Structure(fs2)) => {
                        // Recursively unify embedded structures
                        if let Some(unified_fs) = fs1.unify(fs2) {
                            result.set(attr, FValue::Structure(Box::new(unified_fs)));
                        } else {
                            return None; // Unification failure
                        }
                    },
                    (FValue::Set(set1), FValue::Set(set2)) => {
                        // Combine sets
                        let mut combined = set1.clone();
                        combined.extend(set2.clone());
                        result.set(attr, FValue::Set(combined));
                    },
                    (FValue::Semantic(pred1, args1), FValue::Semantic(pred2, args2)) => {
                        // Semantic forms must match exactly
                        if pred1 != pred2 || args1 != args2 {
                            return None; // Unification failure
                        }
                    },
                    _ => return None, // Mismatched value types
                }
            } else {
                // Only in other, add it to result
                result.set(attr, other_val.clone());
            }
        }
        
        Some(result)
    }
    
    /// Check if this F-structure satisfies a constraint
    pub fn satisfies(&self, constraint: &FConstraint) -> bool {
        match constraint {
            FConstraint::Equation(_, _) => {
                // Equation constraints are checked during construction
                true
            },
            FConstraint::Containment(lhs, rhs) => {
                // For containment, we'd check if rhs is in the set at lhs
                // This is simplified; actual implementation would resolve paths
                if let Some(FValue::Set(set)) = self.get(lhs) {
                    set.iter().any(|val| {
                        match val {
                            FValue::Atomic(s) => s == rhs,
                            _ => false,
                        }
                    })
                } else {
                    false
                }
            },
            FConstraint::Negation(path) => {
                // Check that the path doesn't exist
                !self.has_attribute(path)
            },
            FConstraint::Disjunction(left, right) => {
                // Check if either constraint is satisfied
                self.satisfies(left) || self.satisfies(right)
            },
            FConstraint::DefiningEquation(_, _) => {
                // Defining equations are checked during construction
                true
            },
            FConstraint::ConstrainingEquation(lhs, rhs) => {
                // Check if lhs equals rhs
                if let Some(val) = self.get(lhs) {
                    match val {
                        FValue::Atomic(s) => s == rhs,
                        _ => false,
                    }
                } else {
                    false
                }
            },
            FConstraint::FunctionalUncertainty(_, _) => {
                // Functional uncertainty constraints are checked during construction
                true
            },
            FConstraint::OffPathConstraint(_, _) => {
                // Off-path constraints are checked during construction
                true
            },
            FConstraint::InsideOut(_, _) => {
                // Inside-out functional uncertainty constraints are checked during construction
                true
            },
            FConstraint::SetMembership(_, _) => {
                // Set membership constraints are checked during construction
                true
            },
        }
    }
    
    /// Check if this F-structure is coherent (satisfies uniqueness and completeness)
    pub fn is_coherent(&self) -> bool {
        // Check completeness: if there's a PRED, all its arguments must exist
        if let Some(FValue::Semantic(_, args)) = self.get("PRED") {
            for arg in args {
                if !self.has_attribute(arg) {
                    return false;
                }
            }
        }
        
        // Check uniqueness: no attribute should have multiple conflicting values
        // This is enforced by the HashMap structure, so we don't need to check here
        
        // Check coherence for all embedded F-structures
        for (_, value) in &self.attributes {
            if let FValue::Structure(fs) = value {
                if !fs.is_coherent() {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Evaluate up and down arrows in a path expression relative to this structure
    pub fn resolve_path(&self, path: &str, current: &FStructure) -> Option<FValue> {
        // This is a simplified implementation; a complete one would:
        // 1. Parse the path (e.g., "↑SUBJ PRED")
        // 2. Resolve ↑ to the parent F-structure
        // 3. Resolve ↓ to the current F-structure
        // 4. Follow attribute paths
        
        // For now, just handle simple paths
        if path == "↑" {
            return Some(FValue::Structure(Box::new(self.clone())));
        } else if path == "↓" {
            return Some(FValue::Structure(Box::new(current.clone())));
        } else if path.starts_with("↑") {
            // Follow a path from the parent
            let attr = path.trim_start_matches("↑");
            return self.get(attr).cloned();
        } else if path.starts_with("↓") {
            // Follow a path from the current structure
            let attr = path.trim_start_matches("↓");
            return current.get(attr).cloned();
        } else if !path.contains("↑") && !path.contains("↓") {
            // Direct attribute access
            return self.get(path).cloned();
        } else {
            // Handle complex paths with multiple segments
            let parts: Vec<&str> = path.split_whitespace().collect();
            let mut current_struct = self;
            
            for part in parts {
                if part.starts_with("↑") {
                    let attr = part.trim_start_matches("↑");
                    if attr.is_empty() {
                        continue; // Just the up arrow, stay at current structure
                    }
                    
                    match current_struct.get(attr) {
                        Some(FValue::Structure(fs)) => current_struct = fs,
                        Some(value) => return Some(value.clone()),
                        None => return None,
                    }
                } else if part.starts_with("↓") {
                    let attr = part.trim_start_matches("↓");
                    if attr.is_empty() {
                        current_struct = current;
                    } else {
                        match current.get(attr) {
                            Some(FValue::Structure(fs)) => current_struct = fs,
                            Some(value) => return Some(value.clone()),
                            None => return None,
                        }
                    }
                } else {
                    // Direct attribute
                    match current_struct.get(part) {
                        Some(FValue::Structure(fs)) => current_struct = fs,
                        Some(value) => return Some(value.clone()),
                        None => return None,
                    }
                }
            }
            
            // If we've traversed all parts and ended at a structure, return it
            return Some(FValue::Structure(Box::new(current_struct.clone())));
        }
    }
    
    /// Get a value by following a path of attributes
    pub fn get_by_path(&self, path: &str) -> Option<&FValue> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = self;
        
        for (i, &part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part, return the value
                return current.get(part);
            }
            
            // Not the last part, must be a structure
            match current.get(part) {
                Some(FValue::Structure(fs)) => current = fs,
                _ => return None, // Path doesn't exist or isn't a structure
            }
        }
        
        None
    }
    
    /// Format the F-structure as a string
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
    
    /// Apply functional uncertainty constraints
    /// This handles path equations with wildcards like (↑COMP* OBJ)=↓
    pub fn apply_functional_uncertainty(&mut self, path: &str, value: FValue) -> bool {
        if !path.contains('*') {
            // Regular path, use normal setter
            self.set(path, value);
            return true;
        }
        
        // Parse the path with wildcards
        let parts: Vec<&str> = path.split('.').collect();
        return self.apply_uncertainty_recursive(parts, 0, value);
    }
    
    fn apply_uncertainty_recursive(&mut self, parts: Vec<&str>, index: usize, value: FValue) -> bool {
        if index >= parts.len() {
            // End of path, apply value
            return true;
        }
        
        let part = parts[index];
        if part.ends_with('*') {
            // Handle Kleene star - try different path lengths
            let base_attr = part.trim_end_matches('*');
            
            // Try direct application (0 repetitions)
            let mut success = self.apply_uncertainty_recursive(parts.clone(), index + 1, value.clone());
            
            // Try with one or more repetitions
            if let Some(FValue::Structure(next_fs)) = self.get(base_attr).cloned() {
                let mut fs_copy = (*next_fs).clone();
                if fs_copy.apply_uncertainty_recursive(parts.clone(), index, value.clone()) {
                    self.set(base_attr, FValue::Structure(Box::new(fs_copy)));
                    success = true;
                }
            }
            
            return success;
        } else {
            // Regular attribute
            if index == parts.len() - 1 {
                // Last part of path
                self.set(part, value);
                return true;
            } else {
                // Create embedded structure if needed
                if !self.has_attribute(part) {
                    self.set(part, FValue::Structure(Box::new(FStructure::new(0))));
                }
                
                if let Some(FValue::Structure(fs)) = self.get_mut(part) {
                    return fs.apply_uncertainty_recursive(parts, index + 1, value);
                }
            }
        }
        
        false
    }
    
    /// Handle off-path constraints like (↑ SUBJ NUM) =c (↑ OBJ NUM)
    pub fn apply_off_path_constraint(&mut self, path1: &str, path2: &str) -> bool {
        if let Some(val1) = self.resolve_path(path1, self) {
            if let Some(val2) = self.resolve_path(path2, self) {
                return val1 == val2;
            }
        }
        false
    }
    
    /// Check extended coherence (including semantic forms)
    pub fn is_fully_coherent(&self) -> bool {
        // Basic coherence check
        if !self.is_coherent() {
            return false;
        }
        
        // Check semantic restrictions
        if let Some(FValue::Semantic(_pred, args)) = self.get("PRED") {
            // Check thematic roles and semantic restrictions
            for arg in args {
                if !self.has_attribute(arg) {
                    return false;
                }
                
                // Check semantic type restrictions (simplified)
                match arg.as_str() {
                    "SUBJ" => {
                        // Most predicates require animate subjects
                        if let Some(subj) = self.get(arg) {
                            if let FValue::Structure(fs) = subj {
                                if !fs.has_attribute("ANIM") {
                                    // Subject should have animacy feature
                                    return false;
                                }
                            }
                        }
                    },
                    "OBJ" => {
                        // Some predicates have selectional restrictions on objects
                        // This would be predicate-specific in a full implementation
                    },
                    _ => {}
                }
            }
        }
        
        true
    }
    
    /// Check for extended well-formedness conditions
    pub fn is_well_formed(&self) -> bool {
        // Check coherence
        if !self.is_coherent() {
            return false;
        }
        
        // Check consistency
        if !self.is_consistent() {
            return false;
        }
        
        // Check completeness
        if !self.is_complete() {
            return false;
        }
        
        // Check function-argument biuniqueness
        if !self.satisfies_biuniqueness() {
            return false;
        }
        
        true
    }
    
    /// Check if the F-structure is consistent (no conflicting feature values)
    pub fn is_consistent(&self) -> bool {
        // Already enforced by HashMap, but could add additional checks
        
        // Check embedded structures
        for (_, value) in &self.attributes {
            if let FValue::Structure(fs) = value {
                if !fs.is_consistent() {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Check if the F-structure is complete (all required arguments present)
    pub fn is_complete(&self) -> bool {
        if let Some(FValue::Semantic(_, args)) = self.get("PRED") {
            for arg in args {
                if !self.has_attribute(arg) {
                    return false;
                }
            }
        }
        
        // Check embedded structures
        for (_, value) in &self.attributes {
            if let FValue::Structure(fs) = value {
                if !fs.is_complete() {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Check function-argument biuniqueness
    pub fn satisfies_biuniqueness(&self) -> bool {
        // Each grammatical function can appear at most once
        let mut seen_functions = HashSet::new();
        
        for (attr, _) in &self.attributes {
            // Consider only grammatical functions
            if ["SUBJ", "OBJ", "OBJ2", "COMP", "XCOMP", "OBL"].contains(&attr.as_str()) {
                if seen_functions.contains(attr) {
                    return false;
                }
                seen_functions.insert(attr);
            }
        }
        
        // Check embedded structures
        for (_, value) in &self.attributes {
            if let FValue::Structure(fs) = value {
                if !fs.satisfies_biuniqueness() {
                    return false;
                }
            }
        }
        
        true
    }
}

impl fmt::Display for FStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[")?;
        
        let mut keys: Vec<_> = self.attributes.keys().collect();
        keys.sort(); // Sort for consistent output
        
        for key in keys {
            let value = &self.attributes[key];
            write!(f, "  {}: ", key)?;
            
            match value {
                FValue::Structure(fs) => {
                    // Indent nested structures
                    let fs_str = format!("{}", fs);
                    let indented = fs_str.replace('\n', "\n  ");
                    writeln!(f, "{}", indented)?;
                },
                _ => writeln!(f, "{}", value)?,
            }
        }
        
        write!(f, "]")
    }
}