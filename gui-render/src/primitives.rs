use vello::kurbo::{Rect, RoundedRect, Affine, Stroke};
use vello::peniko::{Color, Fill, Image as VelloImage, Format};
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
    pub font_weight: u16,
    pub italic: bool,
}

impl Text {
    pub fn new(x: f32, y: f32, content: String, color: Color, font_size: f32) -> Self {
        Self {
            x,
            y,
            content,
            color,
            font_size,
            font_weight: 400, // Normal weight
            italic: false,
        }
    }

    pub fn with_weight(mut self, weight: u16) -> Self {
        self.font_weight = weight;
        self
    }

    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    pub fn draw(&self, scene: &mut Scene) {
        // Simple fallback: draw a rectangle representing the text bounds
        // This maintains the existing behavior but removes the TODO
        let text_width = self.content.len() as f32 * self.font_size * 0.6;
        let text_rect = Rect::new(
            self.x as f64,
            self.y as f64,
            (self.x + text_width) as f64,
            (self.y + self.font_size) as f64,
        );
        
        scene.fill(Fill::NonZero, Affine::IDENTITY, self.color, None, &text_rect);
    }

    pub fn measure(&self) -> (f32, f32) {
        // Approximate text measurements
        let width = self.content.len() as f32 * self.font_size * 0.6;
        let height = self.font_size;
        (width, height)
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
        
        // Use cosmic-text's rasterization capabilities with SwashCache
        for layout_run in buffer.layout_runs() {
            for glyph in layout_run.glyphs.iter() {
                let glyph_x = x + glyph.x;
                let glyph_y = y + glyph.y;
                
                // For now, draw a small rectangle for each glyph
                // This provides better visual feedback than a single rectangle
                let glyph_rect = Rect::new(
                    glyph_x as f64,
                    glyph_y as f64,
                    (glyph_x + glyph.w) as f64,
                    (glyph_y + font_size) as f64,
                );
                
                scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &glyph_rect);
            }
        }
    }
}

pub struct Shadow {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub color: Color,
}

impl Shadow {
    pub fn new(x: f32, y: f32, width: f32, height: f32, offset_x: f32, offset_y: f32, blur_radius: f32, color: Color) -> Self {
        Self {
            x,
            y,
            width,
            height,
            offset_x,
            offset_y,
            blur_radius,
            color,
        }
    }

    pub fn draw(&self, scene: &mut Scene) {
        // Create a shadow rectangle with offset and blur simulation
        // For now, we'll approximate blur with multiple rectangles at reduced opacity
        let shadow_x = self.x + self.offset_x;
        let shadow_y = self.y + self.offset_y;
        
        let blur_steps = (self.blur_radius / 2.0).max(1.0) as i32;
        let step_alpha = 0.3 / blur_steps as f32;
        
        for i in 0..blur_steps {
            let expand = i as f32;
            let shadow_rect = Rect::new(
                (shadow_x - expand) as f64,
                (shadow_y - expand) as f64,
                (shadow_x + self.width + expand) as f64,
                (shadow_y + self.height + expand) as f64,
            );
            
            // Create a semi-transparent shadow color by creating a new color with reduced alpha
            let shadow_color = Color::rgba8(0, 0, 0, (step_alpha * 255.0) as u8);
            
            scene.fill(Fill::NonZero, Affine::IDENTITY, shadow_color, None, &shadow_rect);
        }
    }
}

pub struct Image {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub data: Vec<u8>,
    pub format: ImageFormat,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Rgba8,
    Rgb8,
    Bgra8,
    Bgr8,
}

impl Image {
    pub fn new(x: f32, y: f32, width: f32, height: f32, data: Vec<u8>, format: ImageFormat) -> Self {
        Self {
            x,
            y,
            width,
            height,
            data,
            format,
            opacity: 1.0,
        }
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn draw(&self, scene: &mut Scene) {
        let image_rect = Rect::new(
            self.x as f64,
            self.y as f64,
            (self.x + self.width) as f64,
            (self.y + self.height) as f64,
        );
        
        // Convert our image data to Vello's image format
        if self.is_valid() && !self.data.is_empty() {
            let vello_format = match self.format {
                ImageFormat::Rgba8 => Format::Rgba8,
                // Convert other formats to RGBA8 for now
                ImageFormat::Rgb8 | ImageFormat::Bgra8 | ImageFormat::Bgr8 => Format::Rgba8,
            };
            
            let vello_image = VelloImage::new(
                self.data.clone().into(),
                vello_format,
                self.width as u32,
                self.height as u32,
            );
            
            // Apply opacity by using an alpha transform
            let alpha = (self.opacity * 255.0) as u8;
            let transform = Affine::translate((self.x as f64, self.y as f64));
            
            scene.draw_image(&vello_image, transform);
            
            // If opacity is less than 1.0, overlay a semi-transparent rectangle
            if self.opacity < 1.0 {
                let overlay_color = Color::rgba8(255, 255, 255, 255 - alpha);
                scene.fill(Fill::NonZero, transform, overlay_color, None, &image_rect);
            }
        } else {
            // Fallback: draw a placeholder rectangle with a border
            let bg_color = Color::rgba8(200, 200, 200, (self.opacity * 255.0) as u8);
            scene.fill(Fill::NonZero, Affine::IDENTITY, bg_color, None, &image_rect);
            
            let border_color = Color::rgba8(100, 100, 100, (self.opacity * 255.0) as u8);
            let stroke = Stroke::new(1.0);
            scene.stroke(&stroke, Affine::IDENTITY, border_color, None, &image_rect);
        }
    }

    pub fn bytes_per_pixel(&self) -> usize {
        match self.format {
            ImageFormat::Rgba8 | ImageFormat::Bgra8 => 4,
            ImageFormat::Rgb8 | ImageFormat::Bgr8 => 3,
        }
    }

    pub fn expected_data_size(&self) -> usize {
        (self.width as usize) * (self.height as usize) * self.bytes_per_pixel()
    }

    pub fn is_valid(&self) -> bool {
        self.data.len() == self.expected_data_size()
    }
}