pub mod vello_renderer;
pub mod scene_cache;
pub mod primitives;
pub mod batch;

pub use vello_renderer::{VelloRenderer, RenderError};
pub use scene_cache::{SceneCache, CacheKey};
pub use primitives::{Rectangle, Text, TextRenderer, Shadow, Image};
pub use batch::{BatchRenderer, RenderBatch, RenderCommand, BlendMode};

// Re-export common types from dependencies
pub use vello::peniko::Color;
pub use vello::Scene;
pub use vello::kurbo::Affine;