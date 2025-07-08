pub mod vello_renderer;
pub mod scene_cache;
pub mod primitives;

pub use vello_renderer::VelloRenderer;
pub use scene_cache::SceneCache;
pub use primitives::{Rectangle, Text, TextRenderer};

// Re-export common types from dependencies
pub use vello::peniko::Color;
pub use vello::Scene;