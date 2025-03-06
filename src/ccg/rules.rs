//! CCG combinatory rules

use crate::ccg::category::CCGCategory;
use crate::ccg::node::CCGNode;

/// Function for applying CCG rules to derive new categories and nodes
pub trait CCGRule {
    /// Apply this rule to the given nodes and return a new node if successful
    fn apply(&self, left: &CCGNode, right: &CCGNode, use_features: bool) -> Option<CCGNode>;
    
    /// Get the name of this rule
    fn name(&self) -> &str;
}

/// Forward application rule: X/Y Y => X
pub struct ForwardApplication;

impl CCGRule for ForwardApplication {
    fn apply(&self, left: &CCGNode, right: &CCGNode, use_features: bool) -> Option<CCGNode> {
        if let CCGCategory::Forward(x, y) = &left.category {
            if use_features {
                // Try to unify the argument category with the right-hand category
                if let Some(_) = y.unify(&right.category) {
                    // If unification succeeds, create a new node with the resulting category
                    return Some(CCGNode::internal(
                        (**x).clone(),
                        vec![left.clone(), right.clone()],
                        ">",
                    ));
                }
            } else {
                // Simple equality check without unification
                if **y == right.category {
                    let result_category = (**x).clone();
                    return Some(CCGNode::internal(
                        result_category,
                        vec![left.clone(), right.clone()],
                        ">",
                    ));
                }
            }
        }
        None
    }
    
    fn name(&self) -> &str {
        "Forward Application"
    }
}

/// Backward application rule: Y X\Y => X
pub struct BackwardApplication;

impl CCGRule for BackwardApplication {
    fn apply(&self, left: &CCGNode, right: &CCGNode, use_features: bool) -> Option<CCGNode> {
        if let CCGCategory::Backward(x, y) = &right.category {
            if use_features {
                // Try to unify the argument category with the left-hand category
                if let Some(_) = y.unify(&left.category) {
                    // If unification succeeds, create a new node with the resulting category
                    return Some(CCGNode::internal(
                        (**x).clone(),
                        vec![left.clone(), right.clone()],
                        "<",
                    ));
                }
            } else {
                // Simple equality check without unification
                if **y == left.category {
                    let result_category = (**x).clone();
                    return Some(CCGNode::internal(
                        result_category,
                        vec![left.clone(), right.clone()],
                        "<",
                    ));
                }
            }
        }
        None
    }
    
    fn name(&self) -> &str {
        "Backward Application"
    }
}

/// Forward composition rule: X/Y Y/Z => X/Z
pub struct ForwardComposition;

impl CCGRule for ForwardComposition {
    fn apply(&self, left: &CCGNode, right: &CCGNode, use_features: bool) -> Option<CCGNode> {
        if let CCGCategory::Forward(x, y) = &left.category {
            if let CCGCategory::Forward(right_result, right_arg) = &right.category {
                let matches = if use_features {
                    y.unify(right_result).is_some()
                } else {
                    **y == **right_result
                };
                
                if matches {
                    // Construct the result category: X/Z
                    let result = CCGCategory::forward((**x).clone(), (**right_arg).clone());
                    
                    return Some(CCGNode::internal(
                        result,
                        vec![left.clone(), right.clone()],
                        ">B",
                    ));
                }
            }
        }
        None
    }
    
    fn name(&self) -> &str {
        "Forward Composition"
    }
}

/// Backward composition rule: Y\Z X\Y => X\Z
pub struct BackwardComposition;

impl CCGRule for BackwardComposition {
    fn apply(&self, left: &CCGNode, right: &CCGNode, use_features: bool) -> Option<CCGNode> {
        if let CCGCategory::Backward(x, y) = &right.category {
            if let CCGCategory::Backward(left_result, left_arg) = &left.category {
                let matches = if use_features {
                    y.unify(left_result).is_some()
                } else {
                    **y == **left_result
                };
                
                if matches {
                    // Construct the result category: X\Z
                    let result = CCGCategory::backward((**x).clone(), (**left_arg).clone());
                    
                    return Some(CCGNode::internal(
                        result,
                        vec![left.clone(), right.clone()],
                        "<B",
                    ));
                }
            }
        }
        None
    }
    
    fn name(&self) -> &str {
        "Backward Composition"
    }
}

/// Forward type-raising rule: X => T/(T\X)
pub struct ForwardTypeRaising {
    /// Target types for type-raising
    pub targets: Vec<CCGCategory>,
}

impl CCGRule for ForwardTypeRaising {
    fn apply(&self, node: &CCGNode, _right: &CCGNode, _use_features: bool) -> Option<CCGNode> {
        for t in &self.targets {
            // Create T\X
            let backward_cat = CCGCategory::backward(
                t.clone(), 
                node.category.clone()
            );
            
            // Create T/(T\X)
            let new_cat = CCGCategory::forward(
                t.clone(),
                backward_cat
            );
            
            return Some(CCGNode::internal(
                new_cat,
                vec![node.clone()],
                ">T",
            ));
        }
        None
    }
    
    fn name(&self) -> &str {
        "Forward Type Raising"
    }
}

/// Backward type-raising rule: X => T\(T/X)
pub struct BackwardTypeRaising {
    /// Target types for type-raising
    pub targets: Vec<CCGCategory>,
}

impl CCGRule for BackwardTypeRaising {
    fn apply(&self, node: &CCGNode, _right: &CCGNode, _use_features: bool) -> Option<CCGNode> {
        for t in &self.targets {
            // Create T/X
            let forward_cat = CCGCategory::forward(
                t.clone(),
                node.category.clone()
            );
            
            // Create T\(T/X)
            let new_cat = CCGCategory::backward(
                t.clone(),
                forward_cat
            );
            
            return Some(CCGNode::internal(
                new_cat,
                vec![node.clone()],
                "<T",
            ));
        }
        None
    }
    
    fn name(&self) -> &str {
        "Backward Type Raising"
    }
}

/// Function to extract category chain for higher-order composition
pub fn extract_category_chain(
    cat: &CCGCategory, 
    depth: usize, 
    max_depth: usize
) -> Option<(CCGCategory, Vec<(bool, CCGCategory)>)> {
    if depth >= max_depth {
        return None;
    }
    
    match cat {
        CCGCategory::Forward(res, arg) => {
            if depth == 0 {
                // For first level, start building the chain
                Some(((**res).clone(), vec![(true, (**arg).clone())]))
            } else if let Some((base_res, mut args)) = extract_category_chain(res, depth + 1, max_depth) {
                // Add this argument to the chain
                args.push((true, (**arg).clone()));
                Some((base_res, args))
            } else {
                None
            }
        },
        CCGCategory::Backward(res, arg) => {
            if depth == 0 {
                // For first level, start building the chain
                Some(((**res).clone(), vec![(false, (**arg).clone())]))
            } else if let Some((base_res, mut args)) = extract_category_chain(res, depth + 1, max_depth) {
                // Add this argument to the chain
                args.push((false, (**arg).clone()));
                Some((base_res, args))
            } else {
                None
            }
        },
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ccg::category::CCGCategory;
    
    #[test]
    fn test_forward_application() {
        // Test forward application
        let rule = ForwardApplication;
        let np = CCGCategory::np();
        let n = CCGCategory::n();
        let det_cat = CCGCategory::forward(np.clone(), n.clone());
        
        let det_node = CCGNode::leaf("the", det_cat);
        let noun_node = CCGNode::leaf("cat", n.clone());
        
        let result = rule.apply(&det_node, &noun_node, false);
        assert!(result.is_some());
        
        let result_node = result.unwrap();
        assert_eq!(result_node.category, np);
        assert_eq!(result_node.rule, Some(">".to_string()));
    }
    
    #[test]
    fn test_backward_application() {
        // Test backward application
        let rule = BackwardApplication;
        let s = CCGCategory::s();
        let np = CCGCategory::np();
        let verb_cat = CCGCategory::backward(s.clone(), np.clone());
        
        let subj_node = CCGNode::leaf("John", np.clone());
        let verb_node = CCGNode::leaf("sleeps", verb_cat);
        
        let result = rule.apply(&subj_node, &verb_node, false);
        assert!(result.is_some());
        
        let result_node = result.unwrap();
        assert_eq!(result_node.category, s);
        assert_eq!(result_node.rule, Some("<".to_string()));
    }
    
    #[test]
    fn test_forward_composition() {
        // Test forward composition
        let rule = ForwardComposition;
        let s = CCGCategory::s();
        let np = CCGCategory::np();
        let vp = CCGCategory::backward(s.clone(), np.clone());
        
        // Modal verb: (S/VP)/NP
        let modal_cat = CCGCategory::forward(
            CCGCategory::forward(s.clone(), vp.clone()),
            np.clone()
        );
        
        // VP/NP
        let tv_cat = CCGCategory::forward(vp.clone(), np.clone());
        
        let modal_node = CCGNode::leaf("will", modal_cat);
        let tv_node = CCGNode::leaf("chase", tv_cat);
        
        let result = rule.apply(&modal_node, &tv_node, false);
        assert!(result.is_some());
        
        let result_node = result.unwrap();
        // Result should be (S/NP)/NP
        assert_eq!(result_node.rule, Some(">B".to_string()));
    }
    
    #[test]
    fn test_forward_type_raising() {
        // Test forward type raising
        let targets = vec![CCGCategory::s()];
        let rule = ForwardTypeRaising { targets };
        
        let np = CCGCategory::np();
        let np_node = CCGNode::leaf("John", np.clone());
        
        let result = rule.apply(&np_node, &np_node, false); // Second argument is ignored
        assert!(result.is_some());
        
        let result_node = result.unwrap();
        // Result should be S/(S\NP)
        match &result_node.category {
            CCGCategory::Forward(t, y) => {
                assert_eq!(**t, CCGCategory::s());
                match &**y {
                    CCGCategory::Backward(t2, x) => {
                        assert_eq!(**t2, CCGCategory::s());
                        assert_eq!(**x, np);
                    },
                    _ => panic!("Expected backward slash category"),
                }
            },
            _ => panic!("Expected forward slash category"),
        }
    }
}