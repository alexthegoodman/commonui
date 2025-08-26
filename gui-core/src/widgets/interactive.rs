use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, DirtyRegion};
use crate::event::Event;
use winit::event::ElementState;
use gui_reactive::Signal;
use gui_render::primitives::Rectangle;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use vello::peniko::Color;

static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(3000);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

pub struct ButtonWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    label: String,
    state: ButtonState,
    background_color: Color,
    hover_color: Color,
    pressed_color: Color,
    disabled_color: Color,
    border_radius: f32,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    dirty: bool,
}

impl ButtonWidget {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 40.0,
            label: label.into(),
            state: ButtonState::Normal,
            background_color: Color::rgba8(100, 150, 255, 255), // Blue
            hover_color: Color::rgba8(120, 170, 255, 255),      // Lighter blue
            pressed_color: Color::rgba8(80, 130, 235, 255),     // Darker blue
            disabled_color: Color::rgba8(150, 150, 150, 255),   // Gray
            border_radius: 4.0,
            on_click: None,
            dirty: true,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn with_colors(mut self, normal: Color, hover: Color, pressed: Color) -> Self {
        self.background_color = normal;
        self.hover_color = hover;
        self.pressed_color = pressed;
        self.dirty = true;
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self.dirty = true;
        self
    }

    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(callback));
        self
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.dirty = true;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.state = if enabled && self.state == ButtonState::Disabled {
            ButtonState::Normal
        } else if !enabled {
            ButtonState::Disabled
        } else {
            self.state
        };
        self.dirty = true;
    }

    pub fn is_point_inside(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }

    pub fn get_current_color(&self) -> Color {
        match self.state {
            ButtonState::Normal => self.background_color,
            ButtonState::Hovered => self.hover_color,
            ButtonState::Pressed => self.pressed_color,
            ButtonState::Disabled => self.disabled_color,
        }
    }

    pub fn create_background_rectangle(&self) -> Rectangle {
        Rectangle::new(self.x, self.y, self.width, self.height, self.get_current_color())
            .with_border_radius(self.border_radius)
    }
}

impl Widget for ButtonWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        self.dirty = true;
        Ok(())
    }

    fn unmount(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if self.state == ButtonState::Disabled {
            return EventResult::Ignored;
        }

        match event {
            Event::Mouse(mouse_event) => {
                let inside = self.is_point_inside(mouse_event.position.x as f32, mouse_event.position.y as f32);
                let old_state = self.state;

                if mouse_event.button.is_none() {
                    // Mouse move
                    match self.state {
                        ButtonState::Normal => {
                            if inside {
                                self.state = ButtonState::Hovered;
                            }
                        },
                        ButtonState::Hovered => {
                            if !inside {
                                self.state = ButtonState::Normal;
                            }
                        },
                        ButtonState::Pressed => {
                            // Keep pressed state until mouse up
                        },
                        ButtonState::Disabled => {
                            // No state changes when disabled
                        },
                    }
                } else if mouse_event.state == ElementState::Pressed {
                    // Mouse down
                    if inside {
                        self.state = ButtonState::Pressed;
                    }
                } else {
                    // Mouse up
                    if self.state == ButtonState::Pressed {
                        self.state = if inside {
                            if let Some(ref callback) = self.on_click {
                                callback();
                            }
                            ButtonState::Hovered
                        } else {
                            ButtonState::Normal
                        };
                    }
                }

                if old_state != self.state {
                    self.dirty = true;
                    EventResult::Handled
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
        let dirty_region = DirtyRegion {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        };

        Ok(RenderData {
            dirty_regions: vec![dirty_region],
            z_index: 1, // Interactive elements should be above static content
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

pub struct InputWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: Signal<String>,
    placeholder: String,
    is_focused: bool,
    cursor_position: usize,
    background_color: Color,
    border_color: Color,
    focused_border_color: Color,
    text_color: Color,
    placeholder_color: Color,
    border_radius: f32,
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_submit: Option<Box<dyn Fn(&str) + Send + Sync>>,
    dirty: bool,
}

impl InputWidget {
    pub fn new() -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 32.0,
            text: Signal::new(String::new()),
            placeholder: String::new(),
            is_focused: false,
            cursor_position: 0,
            background_color: Color::rgba8(255, 255, 255, 255), // White
            border_color: Color::rgba8(200, 200, 200, 255),     // Light gray
            focused_border_color: Color::rgba8(100, 150, 255, 255), // Blue
            text_color: Color::rgba8(0, 0, 0, 255),             // Black
            placeholder_color: Color::rgba8(150, 150, 150, 255), // Gray
            border_radius: 4.0,
            on_change: None,
            on_submit: None,
            dirty: true,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self.dirty = true;
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        let text_val = text.into();
        self.cursor_position = text_val.len();
        self.text = Signal::new(text_val);
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

    pub fn on_submit<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_submit = Some(Box::new(callback));
        self
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.dirty = true;
    }

    pub fn set_focus(&mut self, focused: bool) {
        if self.is_focused != focused {
            self.is_focused = focused;
            if focused {
                self.cursor_position = self.text.get().len();
            }
            self.dirty = true;
        }
    }

    pub fn is_point_inside(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }

    pub fn get_text(&self) -> String {
        self.text.get()
    }

    pub fn insert_char(&mut self, ch: char) {
        let mut current_text = self.text.get();
        current_text.insert(self.cursor_position, ch);
        self.cursor_position += 1;
        self.text.set(current_text.clone());
        
        if let Some(ref callback) = self.on_change {
            callback(&current_text);
        }
        
        self.dirty = true;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let mut current_text = self.text.get();
            current_text.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.text.set(current_text.clone());
            
            if let Some(ref callback) = self.on_change {
                callback(&current_text);
            }
            
            self.dirty = true;
        }
    }

    pub fn get_border_color(&self) -> Color {
        if self.is_focused {
            self.focused_border_color
        } else {
            self.border_color
        }
    }

    pub fn create_background_rectangle(&self) -> Rectangle {
        Rectangle::new(self.x, self.y, self.width, self.height, self.background_color)
            .with_border_radius(self.border_radius)
            .with_stroke_width(2.0)
    }
}

impl Widget for InputWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        self.dirty = true;
        Ok(())
    }

    fn unmount(&mut self) -> Result<(), WidgetError> {
        self.is_focused = false;
        Ok(())
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::Mouse(mouse_event) => {
                if mouse_event.button.is_some() && mouse_event.state == ElementState::Pressed {
                    // Mouse down
                    let inside = self.is_point_inside(mouse_event.position.x as f32, mouse_event.position.y as f32);
                    self.set_focus(inside);
                    if inside {
                        EventResult::Handled
                    } else {
                        EventResult::Propagate
                    }
                } else {
                    EventResult::Ignored
                }
            },
            Event::Keyboard(keyboard_event) if keyboard_event.state == ElementState::Pressed && self.is_focused => {
                // For now, we'll handle basic keyboard events using scancode
                // In a real implementation, we'd convert scancodes to characters
                match keyboard_event.scancode {
                    14 => { // Backspace
                        self.delete_char();
                        EventResult::Handled
                    },
                    28 => { // Enter
                        if let Some(ref callback) = self.on_submit {
                            callback(&self.text.get());
                        }
                        EventResult::Handled
                    },
                    // For simplicity, we'll skip character input for now
                    // In a real implementation, we'd have proper keyboard handling
                    _ => EventResult::Ignored,
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
        let dirty_region = DirtyRegion {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        };

        Ok(RenderData {
            dirty_regions: vec![dirty_region],
            z_index: 1,
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

pub struct SliderWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    value: Signal<f32>,
    min_value: f32,
    max_value: f32,
    step: f32,
    is_dragging: bool,
    track_color: Color,
    fill_color: Color,
    thumb_color: Color,
    thumb_hover_color: Color,
    thumb_radius: f32,
    on_change: Option<Box<dyn Fn(f32) + Send + Sync>>,
    dirty: bool,
}

impl SliderWidget {
    pub fn new(min_value: f32, max_value: f32) -> Self {
        let initial_value = (min_value + max_value) / 2.0;
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 20.0,
            value: Signal::new(initial_value),
            min_value,
            max_value,
            step: 0.0, // 0.0 means continuous
            is_dragging: false,
            track_color: Color::rgba8(200, 200, 200, 255),      // Light gray
            fill_color: Color::rgba8(100, 150, 255, 255),       // Blue
            thumb_color: Color::rgba8(255, 255, 255, 255),      // White
            thumb_hover_color: Color::rgba8(240, 240, 240, 255), // Light gray
            thumb_radius: 10.0,
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

    pub fn with_value(mut self, value: f32) -> Self {
        let clamped_value = value.clamp(self.min_value, self.max_value);
        self.value = Signal::new(clamped_value);
        self.dirty = true;
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.dirty = true;
    }

    pub fn set_value(&mut self, value: f32) {
        let new_value = if self.step > 0.0 {
            ((value - self.min_value) / self.step).round() * self.step + self.min_value
        } else {
            value
        };
        
        let clamped_value = new_value.clamp(self.min_value, self.max_value);
        
        if (self.value.get() - clamped_value).abs() > f32::EPSILON {
            self.value.set(clamped_value);
            if let Some(ref callback) = self.on_change {
                callback(clamped_value);
            }
            self.dirty = true;
        }
    }

    pub fn get_value(&self) -> f32 {
        self.value.get()
    }

    pub fn value_to_position(&self, value: f32) -> f32 {
        let progress = (value - self.min_value) / (self.max_value - self.min_value);
        self.x + progress * self.width
    }

    pub fn position_to_value(&self, x: f32) -> f32 {
        let progress = ((x - self.x) / self.width).clamp(0.0, 1.0);
        self.min_value + progress * (self.max_value - self.min_value)
    }

    pub fn get_thumb_position(&self) -> f32 {
        self.value_to_position(self.value.get())
    }

    pub fn is_point_on_thumb(&self, x: f32, y: f32) -> bool {
        let thumb_x = self.get_thumb_position();
        let thumb_y = self.y + self.height / 2.0;
        let dx = x - thumb_x;
        let dy = y - thumb_y;
        (dx * dx + dy * dy).sqrt() <= self.thumb_radius
    }

    pub fn is_point_on_track(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }

    pub fn create_track_rectangle(&self) -> Rectangle {
        let track_height = 4.0;
        Rectangle::new(
            self.x,
            self.y + (self.height - track_height) / 2.0,
            self.width,
            track_height,
            self.track_color,
        ).with_border_radius(2.0)
    }

    pub fn create_fill_rectangle(&self) -> Rectangle {
        let track_height = 4.0;
        let fill_width = self.get_thumb_position() - self.x;
        Rectangle::new(
            self.x,
            self.y + (self.height - track_height) / 2.0,
            fill_width,
            track_height,
            self.fill_color,
        ).with_border_radius(2.0)
    }
}

impl Widget for SliderWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        self.dirty = true;
        Ok(())
    }

    fn unmount(&mut self) -> Result<(), WidgetError> {
        self.is_dragging = false;
        Ok(())
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::Mouse(mouse_event) => {
                let x_f32 = mouse_event.position.x as f32;
                let y_f32 = mouse_event.position.y as f32;

                if mouse_event.button.is_none() {
                    // Mouse move
                    if self.is_dragging {
                        let new_value = self.position_to_value(x_f32);
                        self.set_value(new_value);
                        EventResult::Handled
                    } else {
                        EventResult::Ignored
                    }
                } else if mouse_event.state == ElementState::Pressed {
                    // Mouse down
                    if self.is_point_on_thumb(x_f32, y_f32) || self.is_point_on_track(x_f32, y_f32) {
                        self.is_dragging = true;
                        let new_value = self.position_to_value(x_f32);
                        self.set_value(new_value);
                        EventResult::Handled
                    } else {
                        EventResult::Ignored
                    }
                } else {
                    // Mouse up
                    if self.is_dragging {
                        self.is_dragging = false;
                        EventResult::Handled
                    } else {
                        EventResult::Ignored
                    }
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
        let dirty_region = DirtyRegion {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        };

        Ok(RenderData {
            dirty_regions: vec![dirty_region],
            z_index: 1,
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

// Convenience functions for creating interactive widgets
pub fn button(label: impl Into<String>) -> ButtonWidget {
    ButtonWidget::new(label)
}

pub fn input() -> InputWidget {
    InputWidget::new()
}

pub fn slider(min: f32, max: f32) -> SliderWidget {
    SliderWidget::new(min, max)
}