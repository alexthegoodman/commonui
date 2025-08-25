use gui_reactive::{Signal, Computed};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Represents different types of layout invalidations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InvalidationType {
    /// Style properties changed (size, position, flex properties, etc.)
    Style,
    /// Children were added or removed
    Children,
    /// Content size changed (text, image, etc.)
    Content,
    /// Parent size or constraints changed
    Parent,
}

/// Tracks what type of invalidation occurred for a node
#[derive(Debug, Clone)]
pub struct InvalidationInfo {
    pub node_id: u64,
    pub invalidation_type: InvalidationType,
    pub propagate_up: bool,
    pub propagate_down: bool,
}

impl InvalidationInfo {
    pub fn new(node_id: u64, invalidation_type: InvalidationType) -> Self {
        let (propagate_up, propagate_down) = match invalidation_type {
            InvalidationType::Style => (true, false), // Style changes affect ancestors
            InvalidationType::Children => (true, true), // Children changes affect both
            InvalidationType::Content => (true, false), // Content changes affect ancestors
            InvalidationType::Parent => (false, true), // Parent changes affect descendants
        };

        Self {
            node_id,
            invalidation_type,
            propagate_up,
            propagate_down,
        }
    }
}

/// Manages layout invalidation propagation and scheduling
#[derive(Clone)]
pub struct LayoutInvalidationSystem {
    /// Signal that triggers when invalidations need to be processed
    invalidation_signal: Signal<Vec<InvalidationInfo>>,
    
    /// Tracks parent-child relationships for propagation
    parent_child_map: Arc<RwLock<HashMap<u64, Vec<u64>>>>, // parent -> children
    child_parent_map: Arc<RwLock<HashMap<u64, u64>>>, // child -> parent
    
    /// Tracks which nodes are currently dirty and why
    dirty_nodes: Arc<RwLock<HashMap<u64, HashSet<InvalidationType>>>>,
    
    /// Computed signal that provides optimized invalidation batches
    batched_invalidations: Computed<Vec<u64>>,
}

impl LayoutInvalidationSystem {
    pub fn new() -> Self {
        let invalidation_signal = Signal::new(Vec::new());
        let parent_child_map = Arc::new(RwLock::new(HashMap::new()));
        let child_parent_map = Arc::new(RwLock::new(HashMap::new()));
        let dirty_nodes = Arc::new(RwLock::new(HashMap::new()));

        // Create computed signal that batches and optimizes invalidations
        let invalidation_signal_clone = invalidation_signal.clone();
        let parent_child_map_clone = parent_child_map.clone();
        let child_parent_map_clone = child_parent_map.clone();
        let dirty_nodes_clone = dirty_nodes.clone();
        
        let batched_invalidations = Computed::new(move || {
            let invalidations = invalidation_signal_clone.get();
            if invalidations.is_empty() {
                return Vec::new();
            }

            let mut processed_nodes = HashSet::new();
            let mut result = Vec::new();
            
            // Process each invalidation and propagate
            for invalidation in invalidations {
                Self::propagate_invalidation_static(
                    &invalidation,
                    &parent_child_map_clone,
                    &child_parent_map_clone,
                    &dirty_nodes_clone,
                    &mut processed_nodes,
                    &mut result,
                );
            }
            
            // Sort nodes by depth (parents before children) for optimal layout computation
            Self::sort_by_layout_order(&result, &child_parent_map_clone)
        });

        Self {
            invalidation_signal,
            parent_child_map,
            child_parent_map,
            dirty_nodes,
            batched_invalidations,
        }
    }

    /// Register a parent-child relationship
    pub fn register_parent_child(&self, parent_id: u64, child_id: u64) {
        {
            let mut parent_child = self.parent_child_map.write().unwrap();
            parent_child.entry(parent_id).or_insert_with(Vec::new).push(child_id);
        }
        
        {
            let mut child_parent = self.child_parent_map.write().unwrap();
            child_parent.insert(child_id, parent_id);
        }
    }

    /// Unregister a parent-child relationship
    pub fn unregister_parent_child(&self, parent_id: u64, child_id: u64) {
        {
            let mut parent_child = self.parent_child_map.write().unwrap();
            if let Some(children) = parent_child.get_mut(&parent_id) {
                children.retain(|&id| id != child_id);
                if children.is_empty() {
                    parent_child.remove(&parent_id);
                }
            }
        }
        
        {
            let mut child_parent = self.child_parent_map.write().unwrap();
            child_parent.remove(&child_id);
        }
    }

    /// Remove a node from the invalidation system
    pub fn remove_node(&self, node_id: u64) {
        // Get children and parent before removal
        let children = {
            let parent_child = self.parent_child_map.read().unwrap();
            parent_child.get(&node_id).cloned().unwrap_or_default()
        };
        
        let parent = {
            let child_parent = self.child_parent_map.read().unwrap();
            child_parent.get(&node_id).copied()
        };

        // Remove all relationships involving this node
        for child_id in children {
            self.unregister_parent_child(node_id, child_id);
        }
        
        if let Some(parent_id) = parent {
            self.unregister_parent_child(parent_id, node_id);
        }

        // Remove from dirty nodes
        {
            let mut dirty = self.dirty_nodes.write().unwrap();
            dirty.remove(&node_id);
        }
    }

    /// Invalidate a specific node
    pub fn invalidate_node(&self, node_id: u64, invalidation_type: InvalidationType) {
        let invalidation = InvalidationInfo::new(node_id, invalidation_type);
        let mut current_invalidations = self.invalidation_signal.get();
        current_invalidations.push(invalidation);
        self.invalidation_signal.set(current_invalidations);
    }

    /// Get the current batch of nodes that need layout computation
    pub fn get_dirty_nodes(&self) -> Vec<u64> {
        self.batched_invalidations.get()
    }

    /// Clear all invalidations after layout computation
    pub fn clear_invalidations(&self) {
        self.invalidation_signal.set(Vec::new());
        let mut dirty = self.dirty_nodes.write().unwrap();
        dirty.clear();
    }

    /// Get a signal that can be used to react to layout invalidations
    pub fn invalidation_signal(&self) -> Signal<Vec<InvalidationInfo>> {
        self.invalidation_signal.clone()
    }

    /// Get computed signal for optimized dirty nodes
    pub fn dirty_nodes_signal(&self) -> Computed<Vec<u64>> {
        self.batched_invalidations.clone()
    }

    // Static helper method for propagating invalidations
    fn propagate_invalidation_static(
        invalidation: &InvalidationInfo,
        parent_child_map: &Arc<RwLock<HashMap<u64, Vec<u64>>>>,
        child_parent_map: &Arc<RwLock<HashMap<u64, u64>>>,
        dirty_nodes: &Arc<RwLock<HashMap<u64, HashSet<InvalidationType>>>>,
        processed_nodes: &mut HashSet<u64>,
        result: &mut Vec<u64>,
    ) {
        if processed_nodes.contains(&invalidation.node_id) {
            return;
        }

        // Mark this node as dirty
        {
            let mut dirty = dirty_nodes.write().unwrap();
            let entry = dirty.entry(invalidation.node_id).or_insert_with(HashSet::new);
            entry.insert(invalidation.invalidation_type.clone());
        }

        processed_nodes.insert(invalidation.node_id);
        result.push(invalidation.node_id);

        // Propagate up to parents
        if invalidation.propagate_up {
            let parent_id = {
                let child_parent = child_parent_map.read().unwrap();
                child_parent.get(&invalidation.node_id).copied()
            };

            if let Some(parent_id) = parent_id {
                let parent_invalidation = InvalidationInfo::new(parent_id, InvalidationType::Children);
                Self::propagate_invalidation_static(
                    &parent_invalidation,
                    parent_child_map,
                    child_parent_map,
                    dirty_nodes,
                    processed_nodes,
                    result,
                );
            }
        }

        // Propagate down to children
        if invalidation.propagate_down {
            let children = {
                let parent_child = parent_child_map.read().unwrap();
                parent_child.get(&invalidation.node_id).cloned().unwrap_or_default()
            };

            for child_id in children {
                let child_invalidation = InvalidationInfo::new(child_id, InvalidationType::Parent);
                Self::propagate_invalidation_static(
                    &child_invalidation,
                    parent_child_map,
                    child_parent_map,
                    dirty_nodes,
                    processed_nodes,
                    result,
                );
            }
        }
    }

    // Sort nodes by layout computation order (parents before children)
    fn sort_by_layout_order(
        nodes: &[u64],
        child_parent_map: &Arc<RwLock<HashMap<u64, u64>>>,
    ) -> Vec<u64> {
        let mut nodes_with_depth: Vec<(u64, usize)> = nodes.iter().map(|&node_id| {
            let depth = Self::calculate_depth(node_id, child_parent_map);
            (node_id, depth)
        }).collect();

        // Sort by depth (shallower first, then by node_id for consistency)
        nodes_with_depth.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        
        nodes_with_depth.into_iter().map(|(node_id, _)| node_id).collect()
    }

    fn calculate_depth(node_id: u64, child_parent_map: &Arc<RwLock<HashMap<u64, u64>>>) -> usize {
        let child_parent = child_parent_map.read().unwrap();
        let mut current_id = node_id;
        let mut depth = 0;

        while let Some(&parent_id) = child_parent.get(&current_id) {
            depth += 1;
            current_id = parent_id;
        }

        depth
    }
}

impl Default for LayoutInvalidationSystem {
    fn default() -> Self {
        Self::new()
    }
}