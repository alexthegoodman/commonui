use vello::Scene;
use std::collections::HashMap;

pub struct SceneCache {
    cached_scenes: HashMap<u64, Scene>,
    frame_count: u64,
}

impl SceneCache {
    pub fn new() -> Self {
        Self {
            cached_scenes: HashMap::new(),
            frame_count: 0,
        }
    }

    pub fn get_or_create_scene<F>(&mut self, key: u64, create_fn: F) -> &mut Scene
    where
        F: FnOnce() -> Scene,
    {
        self.cached_scenes.entry(key).or_insert_with(create_fn)
    }

    pub fn clear_cache(&mut self) {
        self.cached_scenes.clear();
    }

    pub fn next_frame(&mut self) {
        self.frame_count += 1;
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}