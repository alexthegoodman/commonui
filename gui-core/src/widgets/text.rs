use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, DirtyRegion, WidgetUpdateContext};
use crate::event::Event;
use gui_reactive::Signal;
use gui_render::primitives::{Text as TextPrimitive, TextRenderer, Shadow};
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
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
    shadow: Option<Shadow>,
    pub dirty: bool,
    text_renderer: Option<TextRenderer>,
    // Shared dirty flag that reactive signals can set
    reactive_dirty: Arc<RwLock<bool>>,
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
            shadow: None,
            dirty: true,
            text_renderer: None,
            reactive_dirty: Arc::new(RwLock::new(false)),
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

    pub fn with_shadow(mut self, offset_x: f32, offset_y: f32, blur_radius: f32, color: Color) -> Self {
        // Calculate approximate text dimensions for shadow
        let approx_width = 100.0; // Will be updated when positioned
        let approx_height = 20.0;
        self.shadow = Some(Shadow::new(self.x, self.y, approx_width, approx_height, offset_x, offset_y, blur_radius, color));
        self.dirty = true;
        self
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        if self.x != x || self.y != y {
            self.x = x;
            self.y = y;
            self.dirty = true;
        }
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
        // println!("text prim {:?} {:?}", self.x, self.y);
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

    pub fn create_shadow(&self) -> Option<Shadow> {
        self.shadow.as_ref().map(|shadow| {
            // Use approximate text dimensions for shadow
            let width = self.content.get().len() as f32 * self.font_size.get() * 0.6;
            let height = self.font_size.get();
            Shadow::new(self.x, self.y, width, height, 
                       shadow.offset_x, shadow.offset_y, shadow.blur_radius, shadow.color)
        })
    }
}

impl Widget for TextWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        self.text_renderer = Some(TextRenderer::new());
        self.dirty = true;
        
        // Setup reactive bindings
        let reactive_dirty = self.reactive_dirty.clone();
        self.content.subscribe_fn(move |_| {
            if let Ok(mut dirty) = reactive_dirty.write() {
                *dirty = true;
            }
        });

        let reactive_dirty = self.reactive_dirty.clone();
        self.color.subscribe_fn(move |_| {
            if let Ok(mut dirty) = reactive_dirty.write() {
                *dirty = true;
            }
        });

        let reactive_dirty = self.reactive_dirty.clone();
        self.font_size.subscribe_fn(move |_| {
            if let Ok(mut dirty) = reactive_dirty.write() {
                *dirty = true;
            }
        });

        Ok(())
    }

    fn unmount(&mut self) -> Result<(), WidgetError> {
        self.text_renderer = None;
        Ok(())
    }

    fn update(&mut self, ctx: &mut dyn WidgetUpdateContext) -> Result<(), WidgetError> {
        // Check if reactive signals have changed
        let mut reactive_changed = false;
        if let Ok(mut reactive_dirty) = self.reactive_dirty.write() {
            if *reactive_dirty {
                reactive_changed = true;
                *reactive_dirty = false;
                self.dirty = true;
            }
        }
        
        // If position or content changed, mark as dirty
        if self.dirty {
            ctx.mark_dirty(self.id);
            // Reset dirty flag after marking
            self.dirty = false;
        }
        
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
        let reactive_dirty = if let Ok(dirty) = self.reactive_dirty.read() {
            *dirty
        } else {
            false
        };
        self.dirty || reactive_dirty
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

pub fn text_signal(content_signal: Signal<String>) -> TextWidget {
    let mut widget = TextWidget::new(content_signal.get());
    widget.content = content_signal;
    widget
}

pub fn text_signal_with_style(content_signal: Signal<String>, style: TextStyle) -> TextWidget {
    let mut widget = TextWidget::new(content_signal.get());
    widget.content = content_signal;
    widget
        .with_color(style.color)
        .with_font_size(style.font_size)
        .with_font_weight(style.font_weight)
        .with_italic(style.italic)
}