//! CCG parse tree nodes

use std::fmt;
use crate::ccg::category::CCGCategory;
use crate::common::ParseNode;

/// A parse tree node for CCG parsing
#[derive(Debug, Clone)]
pub struct CCGNode {
    /// The syntactic category
    pub category: CCGCategory,
    /// The original word if this is a leaf
    pub word: Option<String>,
    /// Child nodes
    pub children: Vec<CCGNode>,
    /// The rule used to derive this node
    pub rule: Option<String>,
}

impl CCGNode {
    /// Create a new leaf node
    pub fn leaf(word: &str, category: CCGCategory) -> Self {
        CCGNode {
            category,
            word: Some(word.to_string()),
            children: vec![],
            rule: None,
        }
    }

    /// Create a new internal node
    pub fn internal(category: CCGCategory, children: Vec<CCGNode>, rule: &str) -> Self {
        CCGNode {
            category,
            word: None,
            children,
            rule: Some(rule.to_string()),
        }
    }
}

impl fmt::Display for CCGNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_tree(node: &CCGNode, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let indent_str = " ".repeat(indent);
            
            if let Some(word) = &node.word {
                writeln!(f, "{}{}[{}]", indent_str, word, node.category)?;
            } else if let Some(rule) = &node.rule {
                writeln!(f, "{}{}[{}]", indent_str, rule, node.category)?;
                for child in &node.children {
                    print_tree(child, indent + 2, f)?;
                }
            }
            
            Ok(())
        }
        
        print_tree(self, 0, f)
    }
}

impl ParseNode for CCGNode {
    type Cat = CCGCategory;
    
    fn category(&self) -> &Self::Cat {
        &self.category
    }
    
    fn word(&self) -> Option<&str> {
        self.word.as_deref()
    }
    
    fn children(&self) -> &[Self] {
        &self.children
    }
    
    fn rule(&self) -> Option<&str> {
        self.rule.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ccg::category::CCGCategory;
    
    #[test]
    fn test_node_creation() {
        let np = CCGCategory::np();
        let n = CCGCategory::n();
        let det_cat = CCGCategory::forward(np.clone(), n.clone());
        
        let det_node = CCGNode::leaf("the", det_cat);
        let noun_node = CCGNode::leaf("cat", n.clone());
        
        let np_node = CCGNode::internal(np, vec![det_node, noun_node], ">");
        
        assert_eq!(np_node.children.len(), 2);
        assert_eq!(np_node.rule, Some(">".to_string()));
    }
    
    #[test]
    fn test_parsenode_trait() {
        let np = CCGCategory::np();
        let node = CCGNode::leaf("the", np);
        
        assert_eq!(node.category(), &CCGCategory::np());
        assert_eq!(node.word(), Some("the"));
        assert!(node.children().is_empty());
        assert_eq!(node.rule(), None);
        assert!(node.is_leaf());
    }
}