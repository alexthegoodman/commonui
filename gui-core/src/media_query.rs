use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaFeature {
    MinWidth(u32),
    MaxWidth(u32),
    MinHeight(u32),
    MaxHeight(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MediaQuery {
    features: Vec<MediaFeature>,
}

impl MediaQuery {
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
        }
    }

    pub fn min_width(mut self, width: u32) -> Self {
        self.features.push(MediaFeature::MinWidth(width));
        self
    }

    pub fn max_width(mut self, width: u32) -> Self {
        self.features.push(MediaFeature::MaxWidth(width));
        self
    }

    pub fn min_height(mut self, height: u32) -> Self {
        self.features.push(MediaFeature::MinHeight(height));
        self
    }

    pub fn max_height(mut self, height: u32) -> Self {
        self.features.push(MediaFeature::MaxHeight(height));
        self
    }

    pub fn matches(&self, viewport: ViewportSize) -> bool {
        self.features.iter().all(|feature| {
            match feature {
                MediaFeature::MinWidth(min_width) => viewport.width >= *min_width as f32,
                MediaFeature::MaxWidth(max_width) => viewport.width <= *max_width as f32,
                MediaFeature::MinHeight(min_height) => viewport.height >= *min_height as f32,
                MediaFeature::MaxHeight(max_height) => viewport.height <= *max_height as f32,
            }
        })
    }
}

pub struct MediaQueryManager {
    viewport: ViewportSize,
    cache: HashMap<MediaQuery, bool>,
}

impl MediaQueryManager {
    pub fn new(viewport: ViewportSize) -> Self {
        Self {
            viewport,
            cache: HashMap::new(),
        }
    }

    pub fn set_viewport(&mut self, viewport: ViewportSize) {
        if self.viewport != viewport {
            self.viewport = viewport;
            self.cache.clear(); // Invalidate cache when viewport changes
        }
    }

    pub fn matches(&mut self, query: &MediaQuery) -> bool {
        if let Some(&cached_result) = self.cache.get(query) {
            return cached_result;
        }
        
        let result = query.matches(self.viewport);
        self.cache.insert(query.clone(), result);
        result
    }

    pub fn viewport(&self) -> ViewportSize {
        self.viewport
    }
}

// Helper functions for creating common media queries
pub fn mobile() -> MediaQuery {
    MediaQuery::new().max_width(767)
}

pub fn tablet() -> MediaQuery {
    MediaQuery::new().min_width(768).max_width(1023)
}

pub fn desktop() -> MediaQuery {
    MediaQuery::new().min_width(1024)
}

pub fn small_height() -> MediaQuery {
    MediaQuery::new().max_height(600)
}

pub fn large_height() -> MediaQuery {
    MediaQuery::new().min_height(800)
}

// Responsive styling trait - can be implemented by widgets that need responsive behavior
pub trait ResponsiveWidget {
    fn apply_responsive_styles(&mut self, ctx: &mut dyn crate::WidgetUpdateContext);
}

// Helper macro for responsive styling
#[macro_export]
macro_rules! responsive_style {
    ($ctx:expr, $query:expr, $apply:block) => {
        if $ctx.media_query_manager().matches(&$query) {
            $apply
        }
    };
}