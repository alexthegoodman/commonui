pub mod reactive_layout;
pub mod layout_cache;
pub mod invalidation;
pub mod layout_patterns;

pub use reactive_layout::{ReactiveLayout, ReactiveLayoutManager};
pub use layout_cache::{LayoutCache, AdvancedLayoutCache, DirtyRegion, CachedLayoutInfo, CacheStats};
pub use invalidation::{LayoutInvalidationSystem, InvalidationType, InvalidationInfo};
pub use layout_patterns::{
    FlexLayoutBuilder, FlexItemBuilder, GridLayoutBuilder, GridItemBuilder, 
    LayoutPatterns, ReactiveLayoutPatterns
};

// Re-export common types from taffy
pub use taffy::{Style, Layout, Size, AvailableSpace, FlexDirection, JustifyContent, AlignItems, Position, Dimension, LengthPercentage, LengthPercentageAuto};