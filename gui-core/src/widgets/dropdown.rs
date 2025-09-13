use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, DirtyRegion, WidgetUpdateContext};
use crate::event::Event;
use crate::element::Element;
use crate::sizing::{Unit, Size};
use winit::event::ElementState;
use gui_reactive::Signal;
use gui_render::primitives::{Rectangle, Shadow, Text};
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use vello::peniko::Color;
use super::container::{Background, BoxWidget, container};
use super::text::text;

static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(4000);

#[derive(Clone, Debug)]
pub struct DropdownOption {
    pub value: String,
    pub label: String,
}

impl DropdownOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

pub struct DropdownWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    selected_value: Signal<String>,
    options: Vec<DropdownOption>,
    is_open: bool,
    is_hovering: bool,
    background: Background,
    hover_background: Background,
    border_color: Color,
    text_color: Color,
    arrow_color: Color,
    border_radius: f32,
    font_size: f32,
    max_height: f32,
    shadow: Option<Shadow>,
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    pub dirty: bool,
}

impl DropdownWidget {
    pub fn new() -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 32.0,
            selected_value: Signal::new(String::new()),
            options: Vec::new(),
            is_open: false,
            is_hovering: false,
            background: Background::Color(Color::rgba8(255, 255, 255, 255)),
            hover_background: Background::Color(Color::rgba8(245, 245, 245, 255)),
            border_color: Color::rgba8(200, 200, 200, 255),
            text_color: Color::rgba8(0, 0, 0, 255),
            arrow_color: Color::rgba8(100, 100, 100, 255),
            border_radius: 4.0,
            font_size: 14.0,
            max_height: 200.0,
            shadow: None,
            on_change: None,
            dirty: true,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn with_options(mut self, options: Vec<DropdownOption>) -> Self {
        self.options = options;
        self.dirty = true;
        self
    }

    pub fn with_selected_value(mut self, value: impl Into<String>) -> Self {
        self.selected_value = Signal::new(value.into());
        self.dirty = true;
        self
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        if self.selected_value.get().is_empty() {
            self.selected_value = Signal::new(placeholder.into());
        }
        self.dirty = true;
        self
    }

    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn on_selection_changed<F>(mut self, callback: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(move |value| callback(value.to_string())));
        self
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        if self.x != x || self.y != y {
            self.x = x;
            self.y = y;
            self.dirty = true;
        }
    }

    pub fn is_point_inside(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }

    pub fn is_point_in_dropdown(&self, x: f32, y: f32) -> bool {
        if !self.is_open {
            return false;
        }
        
        let dropdown_y = self.y + self.height;
        let dropdown_height = (self.options.len() as f32 * self.height).min(self.max_height);
        
        x >= self.x && x <= self.x + self.width &&
        y >= dropdown_y && y <= dropdown_y + dropdown_height
    }

    pub fn get_selected_option(&self) -> Option<&DropdownOption> {
        let selected = self.selected_value.get();
        self.options.iter().find(|opt| opt.value == selected)
    }

    pub fn get_option_at_position(&self, x: f32, y: f32) -> Option<&DropdownOption> {
        if !self.is_point_in_dropdown(x, y) {
            return None;
        }
        
        let dropdown_y = self.y + self.height;
        let option_index = ((y - dropdown_y) / self.height) as usize;
        self.options.get(option_index)
    }

    pub fn create_background_rectangle(&self) -> Rectangle {
        let background = if self.is_hovering && !self.is_open {
            &self.hover_background
        } else {
            &self.background
        };
        
        Rectangle::new_with_brush(self.x, self.y, self.width, self.height, background.to_brush())
            .with_border_radius(self.border_radius)
            .with_stroke_width(1.0)
    }

    pub fn create_dropdown_background(&self) -> Option<Rectangle> {
        if !self.is_open {
            return None;
        }
        
        let dropdown_y = self.y + self.height;
        let dropdown_height = (self.options.len() as f32 * self.height).min(self.max_height);
        
        Some(Rectangle::new_with_brush(
            self.x, 
            dropdown_y, 
            self.width, 
            dropdown_height, 
            self.background.to_brush()
        )
        .with_border_radius(self.border_radius)
        .with_stroke_width(1.0))
    }

    pub fn create_text_primitive(&self) -> Option<Text> {
        let display_text = if let Some(option) = self.get_selected_option() {
            &option.label
        } else {
            &self.selected_value.get()
        };
        
        if !display_text.is_empty() {
            let padding = 8.0;
            let text_x = self.x + padding;
            let text_y = self.y + (self.height / 2.0) + (self.font_size * 0.25);
            
            Some(Text::new(text_x, text_y, display_text.clone(), self.text_color, self.font_size))
        } else {
            None
        }
    }

    pub fn create_arrow_primitive(&self) -> Text {
        let arrow_char = if self.is_open { "▲" } else { "▼" };
        let arrow_x = self.x + self.width - 20.0;
        let arrow_y = self.y + (self.height / 2.0) + (self.font_size * 0.25);
        
        Text::new(arrow_x, arrow_y, arrow_char.to_string(), self.arrow_color, self.font_size)
    }

    pub fn create_option_primitives(&self) -> Vec<(Rectangle, Text)> {
        if !self.is_open {
            return Vec::new();
        }
        
        let mut primitives = Vec::new();
        let dropdown_y = self.y + self.height;
        
        for (i, option) in self.options.iter().enumerate() {
            let option_y = dropdown_y + (i as f32 * self.height);
            let option_rect = Rectangle::new(
                self.x, 
                option_y, 
                self.width, 
                self.height, 
                Color::rgba8(255, 255, 255, 255)
            );
            
            let padding = 8.0;
            let text_x = self.x + padding;
            let text_y = option_y + (self.height / 2.0) + (self.font_size * 0.25);
            let option_text = Text::new(text_x, text_y, option.label.clone(), self.text_color, self.font_size);
            
            primitives.push((option_rect, option_text));
        }
        
        primitives
    }

    pub fn select_option(&mut self, option: &DropdownOption) {
        if self.selected_value.get() != option.value {
            self.selected_value.set(option.value.clone());
            if let Some(ref callback) = self.on_change {
                callback(&option.value);
            }
            self.dirty = true;
        }
        self.is_open = false;
    }

    pub fn toggle_dropdown(&mut self) {
        self.is_open = !self.is_open;
        self.dirty = true;
    }
}

impl Widget for DropdownWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        self.dirty = true;
        Ok(())
    }

    fn unmount(&mut self) -> Result<(), WidgetError> {
        self.is_open = false;
        Ok(())
    }

    fn update(&mut self, ctx: &mut dyn WidgetUpdateContext) -> Result<(), WidgetError> {
        if self.dirty {
            ctx.mark_dirty(self.id);
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::Mouse(mouse_event) => {
                let x = mouse_event.position.x as f32;
                let y = mouse_event.position.y as f32;
                
                if mouse_event.button.is_none() {
                    // Mouse move
                    let was_hovering = self.is_hovering;
                    self.is_hovering = self.is_point_inside(x, y);
                    
                    if was_hovering != self.is_hovering {
                        self.dirty = true;
                    }
                    
                    EventResult::Ignored
                } else if mouse_event.state == ElementState::Pressed {
                    // Mouse down
                    if self.is_point_inside(x, y) {
                        self.toggle_dropdown();
                        EventResult::Handled
                    } else if self.is_point_in_dropdown(x, y) {
                        if let Some(option) = self.get_option_at_position(x, y) {
                            let option_value = option.value.clone();
                            if self.selected_value.get() != option_value {
                                self.selected_value.set(option_value.clone());
                                if let Some(ref callback) = self.on_change {
                                    callback(&option_value);
                                }
                                self.is_open = false;
                                self.dirty = true;
                            }
                        }
                        EventResult::Handled
                    } else if self.is_open {
                        // Click outside closes dropdown
                        self.is_open = false;
                        self.dirty = true;
                        EventResult::Handled
                    } else {
                        EventResult::Ignored
                    }
                } else {
                    EventResult::Ignored
                }
            },
            _ => EventResult::Ignored,
        }
    }

    fn needs_layout(&self) -> bool {
        self.dirty
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn render(&self) -> Result<RenderData, WidgetError> {
        let mut dirty_regions = vec![DirtyRegion {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }];
        
        // Add dropdown area if open
        if self.is_open {
            let dropdown_height = (self.options.len() as f32 * self.height).min(self.max_height);
            dirty_regions.push(DirtyRegion {
                x: self.x,
                y: self.y + self.height,
                width: self.width,
                height: dropdown_height,
            });
        }

        Ok(RenderData {
            dirty_regions,
            z_index: if self.is_open { 10 } else { 1 }, // Higher z-index when open
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

// Convenience function for creating dropdowns
pub fn dropdown() -> DropdownWidget {
    DropdownWidget::new()
}