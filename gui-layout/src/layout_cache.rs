use taffy::{Layout, Size, AvailableSpace};
use taffy::prelude::TaffyMaxContent;
use std::collections::HashMap;

pub struct LayoutCache {
    layouts: HashMap<u64, Layout>,
    invalidated_nodes: Vec<u64>,
    available_space: Size<AvailableSpace>,
}

impl LayoutCache {
    pub fn new() -> Self {
        Self {
            layouts: HashMap::new(),
            invalidated_nodes: Vec::new(),
            available_space: Size::MAX_CONTENT,
        }
    }

    pub fn get_layout(&self, node_id: u64) -> Option<&Layout> {
        self.layouts.get(&node_id)
    }

    pub fn set_layout(&mut self, node_id: u64, layout: Layout) {
        self.layouts.insert(node_id, layout);
    }

    pub fn invalidate_node(&mut self, node_id: u64) {
        if !self.invalidated_nodes.contains(&node_id) {
            self.invalidated_nodes.push(node_id);
        }
    }

    pub fn invalidate_all(&mut self) {
        self.invalidated_nodes.clear();
        self.invalidated_nodes.extend(self.layouts.keys());
    }

    pub fn clear_invalidated(&mut self) {
        self.invalidated_nodes.clear();
    }

    pub fn has_invalidated_nodes(&self) -> bool {
        !self.invalidated_nodes.is_empty()
    }

    pub fn invalidated_nodes(&self) -> &[u64] {
        &self.invalidated_nodes
    }

    pub fn remove_layout(&mut self, node_id: u64) {
        self.layouts.remove(&node_id);
        self.invalidated_nodes.retain(|&id| id != node_id);
    }

    pub fn set_available_space(&mut self, available_space: Size<AvailableSpace>) {
        if self.available_space != available_space {
            self.available_space = available_space;
            self.invalidate_all();
        }
    }

    pub fn available_space(&self) -> Size<AvailableSpace> {
        self.available_space
    }

    pub fn clear(&mut self) {
        self.layouts.clear();
        self.invalidated_nodes.clear();
    }
}