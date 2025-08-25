use taffy::{Layout, Size, AvailableSpace, Style};
use taffy::prelude::TaffyMaxContent;
use std::collections::{HashMap, HashSet};
use gui_reactive::{Signal, Computed};
use std::sync::{Arc, RwLock};
use std::time::Instant;

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

/// Represents a rectangular region that needs to be redrawn
#[derive(Debug, Clone, PartialEq)]
pub struct DirtyRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl DirtyRegion {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_layout(layout: &Layout) -> Self {
        Self::new(layout.location.x, layout.location.y, layout.size.width, layout.size.height)
    }

    /// Check if this region intersects with another
    pub fn intersects(&self, other: &DirtyRegion) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

    /// Merge this region with another, returning a region that encompasses both
    pub fn union(&self, other: &DirtyRegion) -> DirtyRegion {
        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);

        DirtyRegion::new(left, top, right - left, bottom - top)
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }
}

/// Cached layout information with additional metadata
#[derive(Debug, Clone)]
pub struct CachedLayoutInfo {
    pub layout: Layout,
    pub style_hash: u64,
    pub available_space: Size<AvailableSpace>,
    pub last_computed: Instant,
    pub dependencies: HashSet<u64>, // Node IDs this layout depends on
}

impl CachedLayoutInfo {
    pub fn new(layout: Layout, style: &Style, available_space: Size<AvailableSpace>) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Create a simple hash of the style - in practice you'd want a more sophisticated approach
        std::ptr::addr_of!(*style).hash(&mut hasher);
        let style_hash = hasher.finish();

        Self {
            layout,
            style_hash,
            available_space,
            last_computed: Instant::now(),
            dependencies: HashSet::new(),
        }
    }

    /// Check if this cached layout is still valid for the given style and available space
    pub fn is_valid(&self, style: &Style, available_space: Size<AvailableSpace>) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        if self.available_space != available_space {
            return false;
        }

        let mut hasher = DefaultHasher::new();
        std::ptr::addr_of!(*style).hash(&mut hasher);
        let current_hash = hasher.finish();

        self.style_hash == current_hash
    }
}

/// Advanced layout cache with dirty region tracking and optimization
#[derive(Clone)]
pub struct AdvancedLayoutCache {
    /// Cached layout information with metadata
    cache: Arc<RwLock<HashMap<u64, CachedLayoutInfo>>>,
    
    /// Dirty regions that need repainting
    dirty_regions: Signal<Vec<DirtyRegion>>,
    
    /// Optimized dirty regions (merged overlapping regions)
    optimized_dirty_regions: Computed<Vec<DirtyRegion>>,
    
    /// Statistics for cache performance
    cache_stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_invalidations: u64,
    pub dirty_regions_optimized: u64,
    pub average_dirty_region_area: f32,
}

impl AdvancedLayoutCache {
    pub fn new() -> Self {
        let cache = Arc::new(RwLock::new(HashMap::new()));
        let dirty_regions = Signal::new(Vec::new());
        let cache_stats = Arc::new(RwLock::new(CacheStats::default()));

        // Create computed signal that optimizes dirty regions
        let dirty_regions_clone = dirty_regions.clone();
        let cache_stats_clone = cache_stats.clone();
        
        let optimized_dirty_regions = Computed::new(move || {
            let regions = dirty_regions_clone.get();
            let optimized = Self::optimize_dirty_regions(&regions);
            
            // Update stats
            if let Ok(mut stats) = cache_stats_clone.write() {
                stats.dirty_regions_optimized += 1;
                if !optimized.is_empty() {
                    let total_area: f32 = optimized.iter().map(|r| r.area()).sum();
                    stats.average_dirty_region_area = total_area / optimized.len() as f32;
                }
            }
            
            optimized
        });

        Self {
            cache,
            dirty_regions,
            optimized_dirty_regions,
            cache_stats,
        }
    }

    /// Try to get a cached layout, return None if not cached or invalid
    pub fn get_cached_layout(&self, node_id: u64, style: &Style, available_space: Size<AvailableSpace>) -> Option<Layout> {
        let cache = self.cache.read().unwrap();
        
        if let Some(cached_info) = cache.get(&node_id) {
            if cached_info.is_valid(style, available_space) {
                // Cache hit
                if let Ok(mut stats) = self.cache_stats.write() {
                    stats.cache_hits += 1;
                }
                return Some(cached_info.layout);
            }
        }
        
        // Cache miss
        if let Ok(mut stats) = self.cache_stats.write() {
            stats.cache_misses += 1;
        }
        
        None
    }

    /// Cache a computed layout with metadata
    pub fn cache_layout(&self, node_id: u64, layout: Layout, style: &Style, available_space: Size<AvailableSpace>) {
        let cached_info = CachedLayoutInfo::new(layout, style, available_space);
        
        // Check if this layout changed from the previous one
        let layout_changed = {
            let cache = self.cache.read().unwrap();
            cache.get(&node_id)
                .map(|old_info| {
                    // Compare layout components since Layout doesn't implement PartialEq
                    old_info.layout.location != layout.location ||
                    old_info.layout.size != layout.size ||
                    old_info.layout.content_size != layout.content_size ||
                    old_info.layout.border != layout.border ||
                    old_info.layout.padding != layout.padding
                })
                .unwrap_or(true) // No previous layout means it changed
        };

        // If layout changed, add to dirty regions
        if layout_changed {
            let dirty_region = DirtyRegion::from_layout(&layout);
            let mut current_regions = self.dirty_regions.get();
            current_regions.push(dirty_region);
            self.dirty_regions.set(current_regions);
        }

        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(node_id, cached_info);
        }
    }

    /// Invalidate a specific node's cache
    pub fn invalidate_node(&self, node_id: u64) {
        let mut cache = self.cache.write().unwrap();
        
        // Add to dirty regions if the node exists
        if let Some(cached_info) = cache.remove(&node_id) {
            let dirty_region = DirtyRegion::from_layout(&cached_info.layout);
            let mut current_regions = self.dirty_regions.get();
            current_regions.push(dirty_region);
            self.dirty_regions.set(current_regions);

            // Update stats
            if let Ok(mut stats) = self.cache_stats.write() {
                stats.cache_invalidations += 1;
            }
        }
    }

    /// Invalidate all cached layouts
    pub fn invalidate_all(&self) {
        let mut cache = self.cache.write().unwrap();
        
        // Add all layouts to dirty regions
        let dirty_regions: Vec<DirtyRegion> = cache.values()
            .map(|cached_info| DirtyRegion::from_layout(&cached_info.layout))
            .collect();
        
        let dirty_regions_count = dirty_regions.len();
        self.dirty_regions.set(dirty_regions);
        cache.clear();

        // Update stats
        if let Ok(mut stats) = self.cache_stats.write() {
            stats.cache_invalidations += dirty_regions_count as u64;
        }
    }

    /// Get the current dirty regions (optimized)
    pub fn get_dirty_regions(&self) -> Vec<DirtyRegion> {
        self.optimized_dirty_regions.get()
    }

    /// Clear all dirty regions (typically called after rendering)
    pub fn clear_dirty_regions(&self) {
        self.dirty_regions.set(Vec::new());
    }

    /// Get a signal that triggers when dirty regions change
    pub fn dirty_regions_signal(&self) -> Signal<Vec<DirtyRegion>> {
        self.dirty_regions.clone()
    }

    /// Get computed signal for optimized dirty regions
    pub fn optimized_dirty_regions_signal(&self) -> Computed<Vec<DirtyRegion>> {
        self.optimized_dirty_regions.clone()
    }

    /// Get current cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.cache_stats.read().unwrap().clone()
    }

    /// Reset cache statistics
    pub fn reset_stats(&self) {
        let mut stats = self.cache_stats.write().unwrap();
        *stats = CacheStats::default();
    }

    /// Check if there are any cached layouts
    pub fn is_empty(&self) -> bool {
        self.cache.read().unwrap().is_empty()
    }

    /// Get the number of cached layouts
    pub fn len(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// Remove a specific node from the cache
    pub fn remove_node(&self, node_id: u64) {
        self.invalidate_node(node_id);
    }

    /// Clear all cached data
    pub fn clear(&self) {
        self.invalidate_all();
    }

    // Optimize dirty regions by merging overlapping regions
    fn optimize_dirty_regions(regions: &[DirtyRegion]) -> Vec<DirtyRegion> {
        if regions.is_empty() {
            return Vec::new();
        }

        let mut optimized: Vec<DirtyRegion> = Vec::new();
        
        for region in regions {
            let mut merged = false;
            
            // Try to merge with existing regions
            for existing in optimized.iter_mut() {
                if existing.intersects(region) {
                    *existing = existing.union(region);
                    merged = true;
                    break;
                }
            }
            
            if !merged {
                optimized.push(region.clone());
            }
        }

        // Multiple passes might be needed if merging creates new overlaps
        let mut changed = true;
        while changed && optimized.len() > 1 {
            changed = false;
            let mut i = 0;
            
            while i < optimized.len() {
                let mut j = i + 1;
                while j < optimized.len() {
                    if optimized[i].intersects(&optimized[j]) {
                        let merged = optimized[i].union(&optimized[j]);
                        optimized[i] = merged;
                        optimized.remove(j);
                        changed = true;
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }

        optimized
    }
}

impl Default for AdvancedLayoutCache {
    fn default() -> Self {
        Self::new()
    }
}