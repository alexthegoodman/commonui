use taffy::{TaffyTree, NodeId, Layout, Style, Size, AvailableSpace};
use std::collections::HashMap;

pub struct ReactiveLayout {
    taffy: TaffyTree,
    node_map: HashMap<u64, NodeId>,
    layout_cache: HashMap<u64, Layout>,
    dirty_nodes: Vec<u64>,
    root_node: Option<NodeId>,
}

impl ReactiveLayout {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_map: HashMap::new(),
            layout_cache: HashMap::new(),
            dirty_nodes: Vec::new(),
            root_node: None,
        }
    }

    pub fn create_node(&mut self, node_id: u64, style: Style) -> Result<(), taffy::TaffyError> {
        let taffy_node = self.taffy.new_leaf(style)?;
        self.node_map.insert(node_id, taffy_node);
        self.dirty_nodes.push(node_id);
        Ok(())
    }

    pub fn create_node_with_children(&mut self, node_id: u64, style: Style, children: Vec<u64>) -> Result<(), taffy::TaffyError> {
        let child_nodes: Vec<NodeId> = children.iter()
            .filter_map(|&child_id| self.node_map.get(&child_id).copied())
            .collect();
        
        let taffy_node = self.taffy.new_with_children(style, &child_nodes)?;
        self.node_map.insert(node_id, taffy_node);
        self.dirty_nodes.push(node_id);
        Ok(())
    }

    pub fn set_root_node(&mut self, node_id: u64) {
        if let Some(&taffy_node) = self.node_map.get(&node_id) {
            self.root_node = Some(taffy_node);
        }
    }

    pub fn update_node_style(&mut self, node_id: u64, style: Style) -> Result<(), taffy::TaffyError> {
        if let Some(&taffy_node) = self.node_map.get(&node_id) {
            self.taffy.set_style(taffy_node, style)?;
            self.dirty_nodes.push(node_id);
        }
        Ok(())
    }

    pub fn add_child(&mut self, parent_id: u64, child_id: u64) -> Result<(), taffy::TaffyError> {
        if let (Some(&parent_node), Some(&child_node)) = (
            self.node_map.get(&parent_id),
            self.node_map.get(&child_id),
        ) {
            self.taffy.add_child(parent_node, child_node)?;
            self.dirty_nodes.push(parent_id);
        }
        Ok(())
    }

    pub fn remove_child(&mut self, parent_id: u64, child_id: u64) -> Result<(), taffy::TaffyError> {
        if let (Some(&parent_node), Some(&child_node)) = (
            self.node_map.get(&parent_id),
            self.node_map.get(&child_id),
        ) {
            self.taffy.remove_child(parent_node, child_node)?;
            self.dirty_nodes.push(parent_id);
        }
        Ok(())
    }

    pub fn compute_layout(&mut self, available_space: Size<AvailableSpace>) -> Result<(), taffy::TaffyError> {
        if let Some(root_node) = self.root_node {
            self.taffy.compute_layout(root_node, available_space)?;
            
            // Cache layouts for all nodes
            for (&node_id, &taffy_node) in &self.node_map {
                if let Ok(layout) = self.taffy.layout(taffy_node) {
                    self.layout_cache.insert(node_id, *layout);
                }
            }
            
            self.dirty_nodes.clear();
        }
        Ok(())
    }

    pub fn get_layout(&self, node_id: u64) -> Option<&Layout> {
        self.layout_cache.get(&node_id)
    }

    pub fn is_dirty(&self) -> bool {
        !self.dirty_nodes.is_empty()
    }

    pub fn invalidate_node(&mut self, node_id: u64) {
        if !self.dirty_nodes.contains(&node_id) {
            self.dirty_nodes.push(node_id);
        }
    }

    pub fn remove_node(&mut self, node_id: u64) -> Result<(), taffy::TaffyError> {
        if let Some(taffy_node) = self.node_map.remove(&node_id) {
            self.taffy.remove(taffy_node)?;
            self.layout_cache.remove(&node_id);
            self.dirty_nodes.retain(|&id| id != node_id);
        }
        Ok(())
    }
}