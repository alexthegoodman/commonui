use vello::kurbo::{Rect, RoundedRect, Affine};
use vello::peniko::{Color, Fill};
use vello::Scene;

pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: Color,
    pub border_radius: f32,
    pub stroke_width: Option<f32>,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32, color: Color) -> Self {
        Self {
            x,
            y,
            width,
            height,
            color,
            border_radius: 0.0,
            stroke_width: None,
        }
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = Some(width);
        self
    }

    pub fn draw(&self, scene: &mut Scene) {
        let rect = Rect::new(self.x as f64, self.y as f64, (self.x + self.width) as f64, (self.y + self.height) as f64);
        
        if self.border_radius > 0.0 {
            let rounded_rect = RoundedRect::from_rect(rect, self.border_radius as f64);
            scene.fill(Fill::NonZero, Affine::IDENTITY, self.color, None, &rounded_rect);
        } else {
            scene.fill(Fill::NonZero, Affine::IDENTITY, self.color, None, &rect);
        }
    }
}

use cosmic_text::{FontSystem, SwashCache, Buffer, Attrs, Metrics, Shaping};

pub struct Text {
    pub x: f32,
    pub y: f32,
    pub content: String,
    pub color: Color,
    pub font_size: f32,
}

impl Text {
    pub fn new(x: f32, y: f32, content: String, color: Color, font_size: f32) -> Self {
        Self {
            x,
            y,
            content,
            color,
            font_size,
        }
    }

    pub fn draw(&self, scene: &mut Scene) {
        // TODO: Implement proper cosmic-text integration
        // For now, we'll draw a placeholder rectangle
        let placeholder_rect = Rect::new(
            self.x as f64,
            self.y as f64,
            (self.x + self.content.len() as f32 * self.font_size * 0.6) as f64,
            (self.y + self.font_size) as f64,
        );
        
        scene.fill(Fill::NonZero, Affine::IDENTITY, self.color, None, &placeholder_rect);
    }
}

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }

    pub fn render_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: Color, scene: &mut Scene) {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, 400.0, 200.0);
        buffer.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system);
        
        // TODO: Convert cosmic-text layout to Vello paths
        // For now, we'll continue using the placeholder
        let placeholder_rect = Rect::new(
            x as f64,
            y as f64,
            (x + text.len() as f32 * font_size * 0.6) as f64,
            (y + font_size) as f64,
        );
        
        scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &placeholder_rect);
    }
}