use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, DirtyRegion};
use crate::event::Event;
use crate::element::Element;
use gui_render::primitives::Rectangle;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
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
    children: Vec<Element>,
    dirty: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
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
        self.x = x;
        self.y = y;
        self.dirty = true;
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

    fn update(&mut self) -> Result<(), WidgetError> {
        for child in &mut self.children {
            child.update()?;
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

pub struct StackWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
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
        self.x = x;
        self.y = y;
        self.dirty = true;
    }

    pub fn add_child(&mut self, child: Element) {
        self.children.push(child);
        self.dirty = true;
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

    fn update(&mut self) -> Result<(), WidgetError> {
        for child in &mut self.children {
            child.update()?;
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