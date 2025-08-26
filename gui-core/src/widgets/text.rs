use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, DirtyRegion};
use crate::event::Event;
use gui_reactive::Signal;
use gui_render::primitives::{Text as TextPrimitive, TextRenderer};
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use vello::peniko::Color;

static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct TextWidget {
    id: WidgetId,
    content: Signal<String>,
    color: Signal<Color>,
    font_size: Signal<f32>,
    font_weight: Signal<u16>,
    italic: Signal<bool>,
    x: f32,
    y: f32,
    dirty: bool,
    text_renderer: Option<TextRenderer>,
}

impl TextWidget {
    pub fn new(content: String) -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            content: Signal::new(content),
            color: Signal::new(Color::rgba8(0, 0, 0, 255)), // Black by default
            font_size: Signal::new(14.0),
            font_weight: Signal::new(400),
            italic: Signal::new(false),
            x: 0.0,
            y: 0.0,
            dirty: true,
            text_renderer: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Signal::new(color);
        self.dirty = true;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = Signal::new(size);
        self.dirty = true;
        self
    }

    pub fn with_font_weight(mut self, weight: u16) -> Self {
        self.font_weight = Signal::new(weight);
        self.dirty = true;
        self
    }

    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = Signal::new(italic);
        self.dirty = true;
        self
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.dirty = true;
    }

    pub fn set_content(&mut self, content: String) {
        self.content.set(content);
        self.dirty = true;
    }

    pub fn set_color(&mut self, color: Color) {
        self.color.set(color);
        self.dirty = true;
    }

    pub fn get_content(&self) -> String {
        self.content.get()
    }

    pub fn measure_text(&mut self) -> (f32, f32) {
        let text_primitive = TextPrimitive::new(
            self.x,
            self.y,
            self.content.get(),
            self.color.get(),
            self.font_size.get(),
        );
        
        // Ensure we have a text renderer
        if self.text_renderer.is_none() {
            self.text_renderer = Some(TextRenderer::new());
        }
        
        if let Some(renderer) = &mut self.text_renderer {
            text_primitive.measure(renderer.font_system_mut())
        } else {
            // Fallback to approximate measurements
            let width = self.content.get().len() as f32 * self.font_size.get() * 0.6;
            let height = self.font_size.get();
            (width, height)
        }
    }

    pub fn create_text_primitive(&self) -> TextPrimitive {
        TextPrimitive::new(
            self.x,
            self.y,
            self.content.get(),
            self.color.get(),
            self.font_size.get(),
        )
        .with_weight(self.font_weight.get())
        .with_italic(self.italic.get())
    }
}

impl Widget for TextWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        self.text_renderer = Some(TextRenderer::new());
        self.dirty = true;
        Ok(())
    }

    fn unmount(&mut self) -> Result<(), WidgetError> {
        self.text_renderer = None;
        Ok(())
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        // For now, we'll mark as dirty if any updates are requested
        // In a more sophisticated implementation, we would track signal changes
        self.dirty = true;
        Ok(())
    }

    fn handle_event(&mut self, _event: &Event) -> EventResult {
        // Text widgets typically don't handle events unless they're editable
        EventResult::Ignored
    }

    fn needs_layout(&self) -> bool {
        self.dirty
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn render(&self) -> Result<RenderData, WidgetError> {
        // Use approximate measurements for rendering (actual text measurement requires mutable access)
        let width = self.content.get().len() as f32 * self.font_size.get() * 0.6;
        let height = self.font_size.get();
        
        let dirty_region = DirtyRegion {
            x: self.x,
            y: self.y,
            width,
            height,
        };

        Ok(RenderData {
            dirty_regions: vec![dirty_region],
            z_index: 0,
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_id(&self) -> WidgetId {
        self.id
    }
}

#[derive(Clone)]
pub struct TextStyle {
    pub color: Color,
    pub font_size: f32,
    pub font_weight: u16,
    pub italic: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Color::rgba8(0, 0, 0, 255),
            font_size: 14.0,
            font_weight: 400,
            italic: false,
        }
    }
}

impl TextStyle {
    pub fn bold() -> Self {
        Self {
            font_weight: 700,
            ..Default::default()
        }
    }

    pub fn italic() -> Self {
        Self {
            italic: true,
            ..Default::default()
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}

pub fn text(content: impl Into<String>) -> TextWidget {
    TextWidget::new(content.into())
}

pub fn text_with_style(content: impl Into<String>, style: TextStyle) -> TextWidget {
    TextWidget::new(content.into())
        .with_color(style.color)
        .with_font_size(style.font_size)
        .with_font_weight(style.font_weight)
        .with_italic(style.italic)
}