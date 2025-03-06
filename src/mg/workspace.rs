//! Workspaces for Minimalist Grammar

use crate::mg::derivation::DerivationTree;

/// A workspace holding a partially-derived tree
#[derive(Debug, Clone)]
pub struct Workspace {
    /// The tree in this workspace
    pub tree: Option<DerivationTree>,
    /// A unique identifier for the workspace
    pub id: usize,
    /// Whether this workspace is active
    pub active: bool,
}

impl Workspace {
    /// Create a new empty workspace
    pub fn new(id: usize) -> Self {
        Self {
            tree: None,
            id,
            active: true,
        }
    }
    
    /// Create a workspace with a tree
    pub fn with_tree(tree: DerivationTree, id: usize) -> Self {
        Self {
            tree: Some(tree),
            id,
            active: true,
        }
    }
    
    /// Set the tree for this workspace
    pub fn set_tree(&mut self, tree: DerivationTree) {
        self.tree = Some(tree);
    }
    
    /// Clear the tree from this workspace
    pub fn clear(&mut self) {
        self.tree = None;
    }
    
    /// Deactivate this workspace
    pub fn deactivate(&mut self) {
        self.active = false;
    }
    
    /// Activate this workspace
    pub fn activate(&mut self) {
        self.active = true;
    }
    
    /// Check if this workspace is empty
    pub fn is_empty(&self) -> bool {
        self.tree.is_none()
    }
}

/// Collection of workspaces for parallel derivations
#[derive(Debug, Clone)]
pub struct WorkspaceRegistry {
    /// All active workspaces
    pub workspaces: Vec<Workspace>,
    /// Next workspace ID
    next_id: usize,
}

impl WorkspaceRegistry {
    /// Create a new workspace registry
    pub fn new() -> Self {
        Self {
            workspaces: Vec::new(),
            next_id: 0,
        }
    }
    
    /// Create a new empty workspace
    pub fn new_workspace(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        self.workspaces.push(Workspace::new(id));
        id
    }
    
    /// Add a tree to a workspace
    pub fn add_tree(&mut self, workspace_id: usize, tree: DerivationTree) -> bool {
        if let Some(workspace) = self.workspaces.iter_mut().find(|w| w.id == workspace_id && w.active) {
            workspace.tree = Some(tree);
            true
        } else {
            false
        }
    }
    
    /// Get a tree from a workspace
    pub fn get_tree(&self, workspace_id: usize) -> Option<&DerivationTree> {
        if let Some(workspace) = self.workspaces.iter().find(|w| w.id == workspace_id && w.active) {
            workspace.tree.as_ref()
        } else {
            None
        }
    }
    
    /// Get a mutable reference to a tree in a workspace
    pub fn get_tree_mut(&mut self, workspace_id: usize) -> Option<&mut DerivationTree> {
        if let Some(workspace) = self.workspaces.iter_mut().find(|w| w.id == workspace_id && w.active) {
            workspace.tree.as_mut()
        } else {
            None
        }
    }
    
    /// Get all active workspaces
    pub fn get_active_workspaces(&self) -> Vec<usize> {
        self.workspaces
            .iter()
            .filter(|w| w.active)
            .map(|w| w.id)
            .collect()
    }
    
    /// Get all active workspaces that contain a tree
    pub fn get_active_workspaces_with_trees(&self) -> Vec<usize> {
        self.workspaces
            .iter()
            .filter(|w| w.active && w.tree.is_some())
            .map(|w| w.id)
            .collect()
    }
    
    /// Deactivate a workspace
    pub fn deactivate(&mut self, workspace_id: usize) {
        if let Some(workspace) = self.workspaces.iter_mut().find(|w| w.id == workspace_id) {
            workspace.active = false;
        }
    }
    
    /// Activate a workspace
    pub fn activate(&mut self, workspace_id: usize) {
        if let Some(workspace) = self.workspaces.iter_mut().find(|w| w.id == workspace_id) {
            workspace.active = true;
        }
    }
    
    /// Transfer a tree from one workspace to another
    pub fn transfer_tree(&mut self, from_id: usize, to_id: usize) -> bool {
        if let Some(from_tree) = self.get_tree(from_id).cloned() {
            self.add_tree(to_id, from_tree);
            
            // Remove tree from source workspace
            if let Some(workspace) = self.workspaces.iter_mut().find(|w| w.id == from_id) {
                workspace.tree = None;
            }
            
            true
        } else {
            false
        }
    }
    
    /// Create a copy of a tree in a new workspace
    pub fn copy_tree(&mut self, from_id: usize) -> Option<usize> {
        if let Some(tree) = self.get_tree(from_id).cloned() {
            let new_id = self.new_workspace();
            self.add_tree(new_id, tree);
            Some(new_id)
        } else {
            None
        }
    }
    
    /// Merge two workspaces into a single workspace
    pub fn merge_workspaces(&mut self, ws1: usize, ws2: usize) -> Option<usize> {
        if let (Some(tree1), Some(tree2)) = (self.get_tree(ws1).cloned(), self.get_tree(ws2).cloned()) {
            // Create a new workspace
            let new_id = self.new_workspace();
            
            // TODO: Implement actual tree merging here
            // For now, just use the first tree
            self.add_tree(new_id, tree1);
            
            // Deactivate the original workspaces
            self.deactivate(ws1);
            self.deactivate(ws2);
            
            Some(new_id)
        } else {
            None
        }
    }
}

impl Default for WorkspaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mg::feature::Feature;
    use crate::mg::lexical_item::LexicalItem;
    
    #[test]
    fn test_workspace_creation() {
        let mut workspace = Workspace::new(1);
        
        assert_eq!(workspace.id, 1);
        assert!(workspace.active);
        assert!(workspace.is_empty());
        
        // Create a simple tree
        let item = LexicalItem::new("the", vec![
            Feature::Categorial("D".to_string()),
        ]);
        
        let tree = DerivationTree::leaf(item, 0);
        
        workspace.set_tree(tree.clone());
        
        assert!(!workspace.is_empty());
        assert_eq!(workspace.tree.as_ref().unwrap().chain.head.phonetic_form, "the");
        
        // Test with_tree constructor
        let workspace2 = Workspace::with_tree(tree.clone(), 2);
        
        assert_eq!(workspace2.id, 2);
        assert!(!workspace2.is_empty());
        assert_eq!(workspace2.tree.as_ref().unwrap().chain.head.phonetic_form, "the");
    }
    
    #[test]
    fn test_workspace_operations() {
        let mut workspace = Workspace::new(1);
        let item = LexicalItem::new("the", vec![
            Feature::Categorial("D".to_string()),
        ]);
        let tree = DerivationTree::leaf(item, 0);
        
        workspace.set_tree(tree);
        assert!(!workspace.is_empty());
        
        workspace.clear();
        assert!(workspace.is_empty());
        
        assert!(workspace.active);
        workspace.deactivate();
        assert!(!workspace.active);
        workspace.activate();
        assert!(workspace.active);
    }
    
    #[test]
    fn test_workspace_registry() {
        let mut registry = WorkspaceRegistry::new();
        
        // Create workspaces
        let id1 = registry.new_workspace();
        let id2 = registry.new_workspace();
        
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(registry.workspaces.len(), 2);
        
        // Add trees
        let item1 = LexicalItem::new("the", vec![Feature::Categorial("D".to_string())]);
        let item2 = LexicalItem::new("cat", vec![Feature::Categorial("N".to_string())]);
        
        let tree1 = DerivationTree::leaf(item1, 0);
        let tree2 = DerivationTree::leaf(item2, 1);
        
        assert!(registry.add_tree(id1, tree1.clone()));
        assert!(registry.add_tree(id2, tree2.clone()));
        
        // Get trees
        if let Some(tree) = registry.get_tree(id1) {
            assert_eq!(tree.chain.head.phonetic_form, "the");
        } else {
            panic!("Expected tree");
        }
        
        if let Some(tree) = registry.get_tree(id2) {
            assert_eq!(tree.chain.head.phonetic_form, "cat");
        } else {
            panic!("Expected tree");
        }
        
        // Test active workspaces
        let active = registry.get_active_workspaces();
        assert_eq!(active.len(), 2);
        assert!(active.contains(&id1));
        assert!(active.contains(&id2));
        
        // Test deactivate/activate
        registry.deactivate(id1);
        let active = registry.get_active_workspaces();
        assert_eq!(active.len(), 1);
        assert!(active.contains(&id2));
        
        registry.activate(id1);
        let active = registry.get_active_workspaces();
        assert_eq!(active.len(), 2);
        
        // Test copy tree
        if let Some(new_id) = registry.copy_tree(id1) {
            if let Some(tree) = registry.get_tree(new_id) {
                assert_eq!(tree.chain.head.phonetic_form, "the");
            } else {
                panic!("Expected tree");
            }
        } else {
            panic!("Expected new workspace ID");
        }
        
        // Test transfer tree
        let id3 = registry.new_workspace();
        assert!(registry.transfer_tree(id1, id3));
        
        if let Some(tree) = registry.get_tree(id3) {
            assert_eq!(tree.chain.head.phonetic_form, "the");
        } else {
            panic!("Expected tree");
        }
        
        // Source workspace should now be empty
        assert!(registry.get_tree(id1).is_none());
    }
}