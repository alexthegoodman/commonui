use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, DirtyRegion, WidgetUpdateContext};
use crate::event::Event;
use crate::element::Element;
use crate::media_query::{MediaQuery, ResponsiveWidget};
use crate::sizing::{Unit, Size};
use gui_render::primitives::{Rectangle, Shadow};
use gui_reactive::signal::Signal;
use std::sync::{Arc, RwLock};
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use vello::peniko::{Color, Gradient, Brush};
use gui_layout::Position;

static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(1000);

/// Background style that can be either a solid color or a gradient
#[derive(Clone, Debug)]
pub enum Background {
    /// Solid color background
    Color(Color),
    /// Gradient background
    Gradient(Gradient),
}

impl Background {
    /// Create a solid color background
    pub fn color(color: Color) -> Self {
        Self::Color(color)
    }
    
    /// Create a gradient background
    pub fn gradient(gradient: Gradient) -> Self {
        Self::Gradient(gradient)
    }
    
    /// Convert to Brush for rendering
    pub fn to_brush(&self) -> Brush {
        match self {
            Self::Color(color) => Brush::Solid(*color),
            Self::Gradient(gradient) => Brush::Gradient(gradient.clone()),
        }
    }
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Self::Color(color)
    }
}

impl From<Gradient> for Background {
    fn from(gradient: Gradient) -> Self {
        Self::Gradient(gradient)
    }
}

pub struct BoxWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    background: Option<Background>,
    border_radius: f32,
    padding: Padding,
    shadow: Option<Shadow>,
    children: Vec<Element>,
    pub dirty: bool,
    // Responsive styling
    responsive_styles: HashMap<MediaQuery, ResponsiveStyle>,
    // Display control signal
    display_signal: Option<Signal<bool>>,
    // Position type for layout (relative, absolute, etc.)
    position: Position,
    // Reactive children support
    reactive_children: Arc<RwLock<Vec<Element>>>,
}

#[derive(Clone, Copy, Debug)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Clone, Debug)]
pub struct ResponsiveStyle {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub background: Option<Background>,
    pub padding: Option<Padding>,
    pub border_radius: Option<f32>,
}

impl Padding {
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn only(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self::all(0.0)
    }
}

impl BoxWidget {
    pub fn new() -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            background: None,
            border_radius: 0.0,
            padding: Padding::default(),
            shadow: None,
            children: Vec::new(),
            dirty: true,
            responsive_styles: HashMap::new(),
            display_signal: None,
            position: Position::Relative,
            reactive_children: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn with_size_units(mut self, width: Unit, height: Unit) -> Self {
        self.width = width.resolve(800.0);
        self.height = height.resolve(600.0);
        self.dirty = true;
        self
    }

    pub fn with_size_perc(mut self, width: f32, height: f32) -> Self {
        self.width = Unit::Perc(width).resolve(800.0);
        self.height = Unit::Perc(height).resolve(600.0);
        self.dirty = true;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self.dirty = true;
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn with_width_perc(mut self, width: f32) -> Self {
        self.width = Unit::Perc(width).resolve(800.0);
        self.dirty = true;
        self
    }

    pub fn with_height_perc(mut self, height: f32) -> Self {
        self.height = Unit::Perc(height).resolve(600.0);
        self.dirty = true;
        self
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background = Some(Background::Color(color));
        self.dirty = true;
        self
    }

    pub fn with_background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self.dirty = true;
        self
    }

    pub fn with_linear_gradient(mut self, gradient: Gradient) -> Self {
        self.background = Some(Background::Gradient(gradient));
        self.dirty = true;
        self
    }

    pub fn with_radial_gradient(mut self, gradient: Gradient) -> Self {
        self.background = Some(Background::Gradient(gradient));
        self.dirty = true;
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self.dirty = true;
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self.dirty = true;
        self
    }

    pub fn with_shadow(mut self, offset_x: f32, offset_y: f32, blur_radius: f32, color: Color) -> Self {
        self.shadow = Some(Shadow::new(self.x, self.y, self.width, self.height, offset_x, offset_y, blur_radius, color));
        self.dirty = true;
        self
    }

    pub fn with_responsive_style(mut self, media_query: MediaQuery, style: ResponsiveStyle) -> Self {
        self.responsive_styles.insert(media_query, style);
        self.dirty = true;
        self
    }

    pub fn with_child(mut self, child: Element) -> Self {
        self.children.push(child);
        self.dirty = true;
        self
    }

    pub fn with_children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self.dirty = true;
        self
    }

    pub fn with_reactive_children<T, F>(mut self, signal: Signal<T>, builder: F) -> Self 
    where 
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> Vec<Element> + Send + Sync + 'static,
    {
        // Build initial children
        let initial_value = signal.get();
        let initial_children = builder(&initial_value);
        self.children = initial_children;
        
        // Store builder and signal for updates
        let reactive_children_ref = Arc::clone(&self.reactive_children);
        let builder_arc = Arc::new(builder);
        signal.subscribe_fn(move |new_value| {
            let new_children = builder_arc(new_value);
            if let Ok(mut reactive_children) = reactive_children_ref.write() {
                *reactive_children = new_children;
            }
        });
        
        self.dirty = true;
        self
    }

    pub fn with_display_signal(mut self, signal: Signal<bool>) -> Self {
        self.display_signal = Some(signal);
        self.dirty = true;
        self
    }

    pub fn absolute(mut self) -> Self {
        self.position = Position::Absolute;
        self.dirty = true;
        self
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self.dirty = true;
        self
    }

    pub fn get_position_type(&self) -> Position {
        self.position
    }

    pub fn is_visible(&self) -> bool {
        if let Some(ref signal) = self.display_signal {
            signal.get()
        } else {
            true
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        if self.x != x || self.y != y {
            self.x = x;
            self.y = y;
            self.dirty = true;
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.dirty = true;
    }

    pub fn add_child(&mut self, child: Element) {
        self.children.push(child);
        self.dirty = true;
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }

    pub fn get_children(&self) -> &Vec<Element> {
        &self.children
    }

    pub fn into_container_element(mut self) -> crate::Element {
        let children = std::mem::take(&mut self.children);
        if children.is_empty() {
            crate::Element::new_widget(Box::new(self))
        } else {
            crate::Element::new_container(Box::new(self), children)
        }
    }

    pub fn get_content_area(&self) -> (f32, f32, f32, f32) {
        (
            self.x + self.padding.left,
            self.y + self.padding.top,
            self.width - self.padding.left - self.padding.right,
            self.height - self.padding.top - self.padding.bottom,
        )
    }

    pub fn create_background_rectangle(&self) -> Option<Rectangle> {
        self.background.as_ref().map(|background| {
            let brush = background.to_brush();
            Rectangle::new_with_brush(self.x, self.y, self.width, self.height, brush)
                .with_border_radius(self.border_radius)
        })
    }

    pub fn create_shadow(&self) -> Option<Shadow> {
        self.shadow.as_ref().map(|shadow| {
            Shadow::new(self.x, self.y, self.width, self.height, 
                       shadow.offset_x, shadow.offset_y, shadow.blur_radius, shadow.color)
        })
    }
}

impl Widget for BoxWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        for child in &mut self.children {
            child.mount()?;
        }
        self.dirty = true;
        Ok(())
    }

    fn unmount(&mut self) -> Result<(), WidgetError> {
        for child in &mut self.children {
            child.unmount()?;
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut dyn WidgetUpdateContext) -> Result<(), WidgetError> {
        // Apply responsive styles
        self.apply_responsive_styles(ctx);
        
        // Check if reactive children have updated
        if let Ok(mut reactive_children) = self.reactive_children.write() {
            if !reactive_children.is_empty() {
                self.children = std::mem::take(&mut *reactive_children);
                self.dirty = true;
            }
        }
        
        if self.dirty {
            ctx.mark_dirty(self.id);
        }
        for child in &mut self.children {
            child.update(ctx)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        // BoxWidget doesn't handle events directly, it just allows propagation to children
        // The Element::Container will handle children event processing and visibility checks
        EventResult::Ignored
    }

    fn needs_layout(&self) -> bool {
        // Don't need layout if display signal is false
        if let Some(ref signal) = self.display_signal {
            if !signal.get() {
                return false;
            }
        }
        
        self.dirty || self.children.iter().any(|child| {
            match child {
                Element::Widget(widget) => widget.needs_layout(),
                Element::Container { widget, .. } => widget.needs_layout(),
                Element::Fragment(_) => false,
            }
        })
    }

    fn needs_render(&self) -> bool {
        // Don't need render if display signal is false
        if let Some(ref signal) = self.display_signal {
            if !signal.get() {
                return false;
            }
        }
        
        self.dirty || self.children.iter().any(|child| {
            match child {
                Element::Widget(widget) => widget.needs_render(),
                Element::Container { widget, .. } => widget.needs_render(),
                Element::Fragment(_) => false,
            }
        })
    }

    fn render(&self) -> Result<RenderData, WidgetError> {
        // Don't render if display signal is false
        if let Some(ref signal) = self.display_signal {
            if !signal.get() {
                return Ok(RenderData {
                    dirty_regions: vec![],
                    z_index: 0,
                });
            }
        }
        
        let dirty_region = DirtyRegion {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
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

impl ResponsiveWidget for BoxWidget {
    fn apply_responsive_styles(&mut self, ctx: &mut dyn crate::WidgetUpdateContext) {
        let mut applied_any = false;
        
        // Apply responsive styles based on matching media queries
        for (media_query, style) in &self.responsive_styles.clone() {
            if ctx.media_query_manager().matches(media_query) {
                if let Some(width) = style.width {
                    self.width = width;
                    applied_any = true;
                }
                if let Some(height) = style.height {
                    self.height = height;
                    applied_any = true;
                }
                if let Some(background) = &style.background {
                    self.background = Some(background.clone());
                    applied_any = true;
                }
                if let Some(padding) = style.padding {
                    self.padding = padding;
                    applied_any = true;
                }
                if let Some(border_radius) = style.border_radius {
                    self.border_radius = border_radius;
                    applied_any = true;
                }
            }
        }
        
        if applied_any {
            self.dirty = true;
        }
    }
}

pub struct StackWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    shadow: Option<Shadow>,
    children: Vec<Element>,
   pub dirty: bool,
}

impl StackWidget {
    pub fn new() -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            shadow: None,
            children: Vec::new(),
            dirty: true,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn with_size_units(mut self, width: Unit, height: Unit) -> Self {
        self.width = width.resolve(800.0);
        self.height = height.resolve(600.0);
        self.dirty = true;
        self
    }

    pub fn with_size_perc(mut self, width: f32, height: f32) -> Self {
        self.width = Unit::Perc(width).resolve(800.0);
        self.height = Unit::Perc(height).resolve(600.0);
        self.dirty = true;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self.dirty = true;
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn with_width_perc(mut self, width: f32) -> Self {
        self.width = Unit::Perc(width).resolve(800.0);
        self.dirty = true;
        self
    }

    pub fn with_height_perc(mut self, height: f32) -> Self {
        self.height = Unit::Perc(height).resolve(600.0);
        self.dirty = true;
        self
    }

    pub fn with_child(mut self, child: Element) -> Self {
        self.children.push(child);
        self.dirty = true;
        self
    }

    pub fn with_children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
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

    pub fn add_child(&mut self, child: Element) {
        self.children.push(child);
        self.dirty = true;
    }

    pub fn with_shadow(mut self, offset_x: f32, offset_y: f32, blur_radius: f32, color: Color) -> Self {
        self.shadow = Some(Shadow::new(self.x, self.y, self.width, self.height, offset_x, offset_y, blur_radius, color));
        self.dirty = true;
        self
    }

    pub fn create_shadow(&self) -> Option<Shadow> {
        self.shadow.as_ref().map(|shadow| {
            Shadow::new(self.x, self.y, self.width, self.height, 
                       shadow.offset_x, shadow.offset_y, shadow.blur_radius, shadow.color)
        })
    }
}

impl Widget for StackWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        for child in &mut self.children {
            child.mount()?;
        }
        self.dirty = true;
        Ok(())
    }

    fn unmount(&mut self) -> Result<(), WidgetError> {
        for child in &mut self.children {
            child.unmount()?;
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut dyn WidgetUpdateContext) -> Result<(), WidgetError> {
        if self.dirty {
            ctx.mark_dirty(self.id);
        }
        for child in &mut self.children {
            child.update(ctx)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        // Stack children are layered, so we handle events from top to bottom (reverse order)
        for child in self.children.iter_mut().rev() {
            match child.handle_event(event) {
                EventResult::Handled => return EventResult::Handled,
                EventResult::Propagate => continue,
                EventResult::Ignored => continue,
            }
        }
        EventResult::Ignored
    }

    fn needs_layout(&self) -> bool {
        self.dirty || self.children.iter().any(|child| {
            match child {
                Element::Widget(widget) => widget.needs_layout(),
                Element::Container { widget, .. } => widget.needs_layout(),
                Element::Fragment(_) => false,
            }
        })
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.children.iter().any(|child| {
            match child {
                Element::Widget(widget) => widget.needs_render(),
                Element::Container { widget, .. } => widget.needs_render(),
                Element::Fragment(_) => false,
            }
        })
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

// Convenience functions for creating containers
pub fn container() -> BoxWidget {
    BoxWidget::new()
}

pub fn stack() -> StackWidget {
    StackWidget::new()
}

// Helper functions for creating responsive styles
impl ResponsiveStyle {
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            background: None,
            padding: None,
            border_radius: None,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background = Some(Background::Color(color));
        self
    }

    pub fn with_background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }
}