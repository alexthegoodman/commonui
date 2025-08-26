use vello::Scene;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKey {
    pub widget_id: u64,
    pub content_hash: u64,
    pub size: (u32, u32),
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.widget_id.hash(state);
        self.content_hash.hash(state);
        self.size.hash(state);
    }
}

struct CachedScene {
    scene: Scene,
    last_used: u64,
    dirty: bool,
}

impl CachedScene {
    fn new(scene: Scene, frame: u64) -> Self {
        Self {
            scene,
            last_used: frame,
            dirty: false,
        }
    }

    fn touch(&mut self, frame: u64) {
        self.last_used = frame;
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn is_expired(&self, current_frame: u64, max_age: u64) -> bool {
        current_frame.saturating_sub(self.last_used) > max_age
    }
}

pub struct SceneCache {
    cached_scenes: HashMap<CacheKey, CachedScene>,
    frame_count: u64,
    max_cache_size: usize,
    max_age_frames: u64,
}

impl SceneCache {
    pub fn new() -> Self {
        Self::with_capacity(256)
    }

    pub fn with_capacity(max_cache_size: usize) -> Self {
        Self {
            cached_scenes: HashMap::with_capacity(max_cache_size),
            frame_count: 0,
            max_cache_size,
            max_age_frames: 120, // 2 seconds at 60fps
        }
    }

    pub fn get_or_create_scene<F>(&mut self, key: CacheKey, create_fn: F) -> &Scene
    where
        F: FnOnce() -> Scene,
    {
        // Check if we need to create or recreate the scene
        let needs_creation = if let Some(cached) = self.cached_scenes.get(&key) {
            cached.dirty
        } else {
            true
        };

        if needs_creation {
            let scene = create_fn();
            let cached_scene = CachedScene::new(scene, self.frame_count);
            self.cached_scenes.insert(key.clone(), cached_scene);
            self.evict_if_needed();
        } else {
            // Just update the access time
            if let Some(cached) = self.cached_scenes.get_mut(&key) {
                cached.touch(self.frame_count);
            }
        }
        
        &self.cached_scenes.get(&key).unwrap().scene
    }

    pub fn invalidate(&mut self, key: &CacheKey) {
        if let Some(cached) = self.cached_scenes.get_mut(key) {
            cached.mark_dirty();
        }
    }

    pub fn invalidate_widget(&mut self, widget_id: u64) {
        for (key, cached) in &mut self.cached_scenes {
            if key.widget_id == widget_id {
                cached.mark_dirty();
            }
        }
    }

    pub fn clear_cache(&mut self) {
        self.cached_scenes.clear();
    }

    pub fn next_frame(&mut self) {
        self.frame_count += 1;
        self.cleanup_expired();
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn cache_size(&self) -> usize {
        self.cached_scenes.len()
    }

    pub fn cache_hit_ratio(&self) -> f32 {
        // This would need hit/miss counters for accurate metrics
        // For now, return a placeholder
        0.0
    }

    fn cleanup_expired(&mut self) {
        self.cached_scenes.retain(|_, cached| {
            !cached.is_expired(self.frame_count, self.max_age_frames)
        });
    }

    fn evict_if_needed(&mut self) {
        if self.cached_scenes.len() <= self.max_cache_size {
            return;
        }

        // Simple LRU eviction - remove oldest entries
        let mut entries: Vec<_> = self.cached_scenes.iter().map(|(k, v)| (k.clone(), v.last_used)).collect();
        entries.sort_by_key(|(_, last_used)| *last_used);
        
        let to_remove = entries.len() - self.max_cache_size;
        let keys_to_remove: Vec<_> = entries.iter().take(to_remove).map(|(key, _)| key.clone()).collect();
        
        for key in keys_to_remove {
            self.cached_scenes.remove(&key);
        }
    }
}