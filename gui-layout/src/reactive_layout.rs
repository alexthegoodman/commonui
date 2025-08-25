use taffy::{TaffyTree, NodeId, Layout, Style, Size, AvailableSpace};
use std::collections::HashMap;
use gui_reactive::{Signal, Effect, global_frame_scheduler};
use std::sync::{Arc, RwLock};
use crate::invalidation::{LayoutInvalidationSystem, InvalidationType};

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

/// A reactive wrapper around ReactiveLayout that integrates with the signal system
pub struct ReactiveLayoutManager {
    layout: Arc<RwLock<ReactiveLayout>>,
    layout_signal: Signal<bool>, // Triggers when layout changes
    invalidation_system: LayoutInvalidationSystem, // Handles invalidation propagation
}

impl ReactiveLayoutManager {
    pub fn new() -> Self {
        let layout = Arc::new(RwLock::new(ReactiveLayout::new()));
        let layout_signal = Signal::new(false);
        let invalidation_system = LayoutInvalidationSystem::new();

        // Create an effect that schedules layout computation when nodes are invalidated
        let _layout_clone = layout.clone();
        let layout_signal_clone = layout_signal.clone();
        let dirty_nodes_signal = invalidation_system.dirty_nodes_signal();
        
        Effect::new(move || {
            let dirty_nodes = dirty_nodes_signal.get();
            if !dirty_nodes.is_empty() {
                // Schedule layout computation for next frame
                global_frame_scheduler().schedule_for_next_frame(Box::new(move || {
                    // This will be handled by the layout computation system
                }));
                
                // Mark layout as changed
                layout_signal_clone.set(true);
            }
        });

        Self {
            layout,
            layout_signal,
            invalidation_system,
        }
    }

    /// Create a reactive node with a style signal
    pub fn create_reactive_node(&self, node_id: u64, style_signal: Signal<Style>) -> Result<(), taffy::TaffyError> {
        let initial_style = style_signal.get();
        
        // Create the node with initial style
        {
            let mut layout = self.layout.write().unwrap();
            layout.create_node(node_id, initial_style)?;
        }

        // Create an effect that updates the node style when the signal changes
        let layout_clone = self.layout.clone();
        let invalidation_system_clone = self.invalidation_system.clone();
        Effect::new(move || {
            let new_style = style_signal.get();
            if let Ok(mut layout) = layout_clone.write() {
                let _ = layout.update_node_style(node_id, new_style);
                
                // Trigger invalidation through the invalidation system
                invalidation_system_clone.invalidate_node(node_id, InvalidationType::Style);
            }
        });

        Ok(())
    }

    /// Create a reactive node with children and a style signal
    pub fn create_reactive_node_with_children(
        &self,
        node_id: u64,
        style_signal: Signal<Style>,
        children: Vec<u64>
    ) -> Result<(), taffy::TaffyError> {
        let initial_style = style_signal.get();
        
        // Create the node with initial style and children
        {
            let mut layout = self.layout.write().unwrap();
            layout.create_node_with_children(node_id, initial_style, children)?;
        }

        // Create an effect that updates the node style when the signal changes
        let layout_clone = self.layout.clone();
        let invalidation_system_clone = self.invalidation_system.clone();
        Effect::new(move || {
            let new_style = style_signal.get();
            if let Ok(mut layout) = layout_clone.write() {
                let _ = layout.update_node_style(node_id, new_style);
                
                // Trigger invalidation through the invalidation system
                invalidation_system_clone.invalidate_node(node_id, InvalidationType::Style);
            }
        });

        Ok(())
    }

    /// Set the root node for layout computation
    pub fn set_root_node(&self, node_id: u64) {
        let mut layout = self.layout.write().unwrap();
        layout.set_root_node(node_id);
    }

    /// Add a child to a parent node
    pub fn add_child(&self, parent_id: u64, child_id: u64) -> Result<(), taffy::TaffyError> {
        let mut layout = self.layout.write().unwrap();
        layout.add_child(parent_id, child_id)?;
        
        // Register parent-child relationship and trigger invalidation
        self.invalidation_system.register_parent_child(parent_id, child_id);
        self.invalidation_system.invalidate_node(parent_id, InvalidationType::Children);
        
        Ok(())
    }

    /// Remove a child from a parent node
    pub fn remove_child(&self, parent_id: u64, child_id: u64) -> Result<(), taffy::TaffyError> {
        let mut layout = self.layout.write().unwrap();
        layout.remove_child(parent_id, child_id)?;
        
        // Unregister parent-child relationship and trigger invalidation
        self.invalidation_system.unregister_parent_child(parent_id, child_id);
        self.invalidation_system.invalidate_node(parent_id, InvalidationType::Children);
        
        Ok(())
    }

    /// Compute layout for the current frame
    pub fn compute_layout(&self, available_space: Size<AvailableSpace>) -> Result<(), taffy::TaffyError> {
        let mut layout = self.layout.write().unwrap();
        layout.compute_layout(available_space)?;
        
        // Clear invalidations after successful layout computation
        self.invalidation_system.clear_invalidations();
        
        Ok(())
    }

    /// Get layout for a specific node
    pub fn get_layout(&self, node_id: u64) -> Option<Layout> {
        let layout = self.layout.read().unwrap();
        layout.get_layout(node_id).copied()
    }

    /// Check if layout needs recomputation
    pub fn is_dirty(&self) -> bool {
        let layout = self.layout.read().unwrap();
        layout.is_dirty()
    }

    /// Manually invalidate a node
    pub fn invalidate_node(&self, node_id: u64, invalidation_type: InvalidationType) {
        self.invalidation_system.invalidate_node(node_id, invalidation_type);
    }

    /// Remove a node completely
    pub fn remove_node(&self, node_id: u64) -> Result<(), taffy::TaffyError> {
        // Remove from invalidation system first
        self.invalidation_system.remove_node(node_id);
        
        let mut layout = self.layout.write().unwrap();
        layout.remove_node(node_id)
    }

    /// Get a signal that triggers when layout changes
    pub fn layout_changed_signal(&self) -> Signal<bool> {
        self.layout_signal.clone()
    }

    /// Get the underlying layout for direct access (use carefully)
    pub fn get_layout_manager(&self) -> Arc<RwLock<ReactiveLayout>> {
        self.layout.clone()
    }
}