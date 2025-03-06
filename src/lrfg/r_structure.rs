//! Realizational structure (R-structure) for Lexical-Realizational Functional Grammar
//!
//! R-structure in LRFG mediates between f-structure and phonological form.
//! It consists of feature bundles that are realized by vocabulary items.

use std::fmt;
use std::collections::HashSet;

/// A feature in the R-structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RFeature {
    /// Name of the feature
    pub name: String,
    /// Value of the feature
    pub value: String,
}

impl RFeature {
    /// Create a new R-structure feature
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

impl fmt::Display for RFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}]", self.name, self.value)
    }
}

/// A node in the R-structure
#[derive(Debug, Clone)]
pub struct RNode {
    /// Unique identifier for this node
    pub id: usize,
    /// Features of this node
    pub features: HashSet<RFeature>,
    /// Children of this node
    pub children: Vec<RNode>,
    /// Phonological form (if realized)
    pub form: Option<String>,
}

impl RNode {
    /// Create a new R-structure node
    pub fn new(id: usize) -> Self {
        Self {
            id,
            features: HashSet::new(),
            children: Vec::new(),
            form: None,
        }
    }
    
    /// Add a feature to this node
    pub fn add_feature(&mut self, name: &str, value: &str) {
        self.features.insert(RFeature::new(name, value));
    }
    
    /// Check if this node has a specific feature
    pub fn has_feature(&self, name: &str, value: &str) -> bool {
        self.features.contains(&RFeature::new(name, value))
    }
    
    /// Check if this node has a feature with a specific name
    pub fn has_feature_name(&self, name: &str) -> bool {
        self.features.iter().any(|f| f.name == name)
    }
    
    /// Get the value of a feature
    pub fn get_feature_value(&self, name: &str) -> Option<&str> {
        self.features.iter()
            .find(|f| f.name == name)
            .map(|f| f.value.as_str())
    }
    
    /// Add a child node
    pub fn add_child(&mut self, child: RNode) {
        self.children.push(child);
    }
    
    /// Set the phonological form
    pub fn set_form(&mut self, form: &str) {
        self.form = Some(form.to_string());
    }
    
    /// Get all nodes in the R-structure
    pub fn get_all_nodes(&mut self) -> Vec<&mut RNode> {
        let mut result = Vec::new();
        // First collect self
        result.push(self);
        
        // Process children one by one to avoid multiple mutable borrows
        for i in 0..self.children.len() {
            if let Some(child) = self.children.get_mut(i) {
                let mut child_nodes = child.get_all_nodes();
                result.append(&mut child_nodes);
            }
        }
        
        result
    }
    
    /// Apply a function to all nodes in the R-structure
    pub fn for_each_node<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut RNode),
    {
        f(self);
        
        let indices: Vec<usize> = (0..self.children.len()).collect();
        
        for i in indices {
            if let Some(child) = self.children.get_mut(i) {
                child.for_each_node(f);
            }
        }
    }
}

impl fmt::Display for RNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_node(node: &RNode, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let indent_str = " ".repeat(indent);
            
            // Print features
            write!(f, "{}Node {}:", indent_str, node.id)?;
            
            if !node.features.is_empty() {
                let features: Vec<_> = node.features.iter().collect();
                for feature in features {
                    write!(f, " {}", feature)?;
                }
            }
            
            if let Some(form) = &node.form {
                write!(f, " → \"{}\"", form)?;
            }
            
            writeln!(f)?;
            
            // Print children
            for child in &node.children {
                print_node(child, indent + 2, f)?;
            }
            
            Ok(())
        }
        
        print_node(self, 0, f)
    }
}

/// Complete R-structure
#[derive(Debug, Clone)]
pub struct RStructure {
    /// Root node of the R-structure
    pub root: RNode,
    /// Next available node ID
    next_id: usize,
}

impl RStructure {
    /// Create a new R-structure
    pub fn new() -> Self {
        let root = RNode::new(0);
        Self {
            root,
            next_id: 1,
        }
    }
    
    /// Create a new node and get its ID
    pub fn new_node(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    /// Get a node by ID
    pub fn get_node(&self, id: usize) -> Option<&RNode> {
        self.find_node(&self.root, id)
    }
    
    /// Get a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut RNode> {
        if self.root.id == id {
            return Some(&mut self.root);
        }
        
        self.find_node_mut_in_children(&mut self.root.children, id)
    }
    
    /// Find a node by ID (helper function)
    fn find_node<'a>(&'a self, node: &'a RNode, id: usize) -> Option<&'a RNode> {
        if node.id == id {
            return Some(node);
        }
        
        for child in &node.children {
            if let Some(found) = self.find_node(child, id) {
                return Some(found);
            }
        }
        
        None
    }
    
    /// Find a mutable node by ID in a vector of children
    fn find_node_mut_in_children<'a>(&'a mut self, children: &'a mut Vec<RNode>, id: usize) -> Option<&'a mut RNode> {
        for i in 0..children.len() {
            if let Some(child) = children.get_mut(i) {
                if child.id == id {
                    return Some(child);
                }
                
                let child_children = &mut child.children;
                if let Some(found) = self.find_node_mut_in_children(child_children, id) {
                    return Some(found);
                }
            }
        }
        
        None
    }
    
    /// Generate the phonological form from the R-structure
    pub fn realize(&self) -> String {
        fn realize_node(node: &RNode) -> String {
            if let Some(form) = &node.form {
                if node.children.is_empty() {
                    return form.clone();
                } else {
                    let child_forms: Vec<String> = node.children.iter()
                        .map(realize_node)
                        .collect();
                    return format!("{} {}", form, child_forms.join(" "));
                }
            } else if !node.children.is_empty() {
                let child_forms: Vec<String> = node.children.iter()
                    .map(realize_node)
                    .collect();
                return child_forms.join(" ");
            }
            
            String::new()
        }
        
        realize_node(&self.root)
    }
    
    /// Get all nodes in the R-structure
    pub fn get_all_nodes(&mut self) -> Vec<&mut RNode> {
        self.root.get_all_nodes()
    }
    
    /// Apply a function to all nodes in the R-structure
    pub fn for_each_node<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut RNode),
    {
        let mut f_ref = &mut f;
        self.root.for_each_node(f_ref);
    }
}

impl fmt::Display for RStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rfeature() {
        let feature = RFeature::new("num", "sg");
        assert_eq!(feature.name, "num");
        assert_eq!(feature.value, "sg");
        
        let display = format!("{}", feature);
        assert_eq!(display, "[num:sg]");
    }
    
    #[test]
    fn test_rnode() {
        let mut node = RNode::new(1);
        node.add_feature("num", "sg");
        node.add_feature("pers", "3");
        
        assert!(node.has_feature("num", "sg"));
        assert!(node.has_feature_name("pers"));
        assert_eq!(node.get_feature_value("num"), Some("sg"));
        
        node.set_form("cat");
        assert_eq!(node.form, Some("cat".to_string()));
    }
    
    #[test]
    fn test_rstructure() {
        let mut r_structure = RStructure::new();
        
        // Add features to root
        r_structure.root.add_feature("cat", "NP");
        
        // Create child nodes
        let det_id = r_structure.new_node();
        let mut det_node = RNode::new(det_id);
        det_node.add_feature("cat", "Det");
        det_node.set_form("the");
        
        let n_id = r_structure.new_node();
        let mut n_node = RNode::new(n_id);
        n_node.add_feature("cat", "N");
        n_node.add_feature("num", "sg");
        n_node.set_form("cat");
        
        // Add children to root
        r_structure.root.add_child(det_node);
        r_structure.root.add_child(n_node);
        
        // Test node retrieval
        let retrieved = r_structure.get_node(det_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().form, Some("the".to_string()));
        
        // Test realization
        let form = r_structure.realize();
        assert_eq!(form, "the cat");
    }
} 