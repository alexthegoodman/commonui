use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, DirtyRegion, WidgetUpdateContext};
use crate::event::Event;
use crate::element::Element;
use crate::media_query::{MediaQuery, ResponsiveWidget};
use gui_render::primitives::{Rectangle, Shadow};
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use vello::peniko::Color;

static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(1000);

pub struct BoxWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    background_color: Option<Color>,
    border_radius: f32,
    padding: Padding,
    shadow: Option<Shadow>,
    children: Vec<Element>,
    pub dirty: bool,
    // Responsive styling
    responsive_styles: HashMap<MediaQuery, ResponsiveStyle>,
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
    pub background_color: Option<Color>,
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
            background_color: None,
            border_radius: 0.0,
            padding: Padding::default(),
            shadow: None,
            children: Vec::new(),
            dirty: true,
            responsive_styles: HashMap::new(),
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
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
        self.background_color.map(|color| {
            Rectangle::new(self.x, self.y, self.width, self.height, color)
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
        
        if self.dirty {
            ctx.mark_dirty(self.id);
        }
        for child in &mut self.children {
            child.update(ctx)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        for child in &mut self.children {
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
                if let Some(background_color) = style.background_color {
                    self.background_color = Some(background_color);
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
    dirty: bool,
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
            background_color: None,
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
        self.background_color = Some(color);
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