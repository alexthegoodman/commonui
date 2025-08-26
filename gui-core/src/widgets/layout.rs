use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, DirtyRegion};
use crate::event::Event;
use crate::element::Element;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};

static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(2000);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MainAxisAlignment {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrossAxisAlignment {
    Start,
    End,
    Center,
    Stretch,
}

pub struct RowWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    gap: f32,
    children: Vec<Element>,
    dirty: bool,
}

impl RowWidget {
    pub fn new() -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Center,
            gap: 0.0,
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

    pub fn with_main_axis_alignment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self.dirty = true;
        self
    }

    pub fn with_cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self.dirty = true;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
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

    pub fn layout_children(&mut self) {
        if self.children.is_empty() {
            return;
        }

        // For simplicity, assume each child takes equal width
        // In a real implementation, this would use the layout system (Taffy)
        let child_count = self.children.len() as f32;
        let total_gap = self.gap * (child_count - 1.0);
        let available_width = self.width - total_gap;
        let child_width = available_width / child_count;

        let mut current_x = self.x;

        // Apply main axis alignment
        match self.main_axis_alignment {
            MainAxisAlignment::Start => {
                current_x = self.x;
            }
            MainAxisAlignment::End => {
                current_x = self.x + self.width - (child_width * child_count + total_gap);
            }
            MainAxisAlignment::Center => {
                current_x = self.x + (self.width - (child_width * child_count + total_gap)) / 2.0;
            }
            MainAxisAlignment::SpaceBetween | 
            MainAxisAlignment::SpaceAround | 
            MainAxisAlignment::SpaceEvenly => {
                current_x = self.x;
            }
        }

        let num_children = self.children.len();
        let gap = self.gap;
        let row_y = self.y;
        let row_height = self.height;
        let cross_alignment = self.cross_axis_alignment;
        
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_y = match cross_alignment {
                CrossAxisAlignment::Start => row_y,
                CrossAxisAlignment::End => row_y + row_height,
                CrossAxisAlignment::Center => row_y + (row_height / 2.0),
                CrossAxisAlignment::Stretch => row_y,
            };

            // Position the child widget
            Self::position_child_element(child, current_x, child_y, child_width, row_height);
            
            current_x += child_width;
            if i < num_children - 1 {
                current_x += gap;
            }
        }
    }
    
    fn position_child_element(child: &mut Element, x: f32, y: f32, _width: f32, _height: f32) {
        match child {
            Element::Widget(widget) => {
                use crate::widgets::{text::TextWidget, interactive::ButtonWidget, container::BoxWidget};
                
                if let Some(text_widget) = widget.as_any_mut().downcast_mut::<TextWidget>() {
                    text_widget.set_position(x, y);
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    button_widget.set_position(x, y);
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    box_widget.set_position(x, y);
                }
            },
            Element::Container { widget, .. } => {
                use crate::widgets::{text::TextWidget, interactive::ButtonWidget, container::BoxWidget};
                
                if let Some(text_widget) = widget.as_any_mut().downcast_mut::<TextWidget>() {
                    text_widget.set_position(x, y);
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    button_widget.set_position(x, y);
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    box_widget.set_position(x, y);
                }
            },
            Element::Fragment(_) => {
                // Fragments don't have a position
            }
        }
    }
    
    pub fn into_container_element(mut self) -> crate::Element {
        let children = std::mem::take(&mut self.children);
        crate::Element::new_container(Box::new(self), children)
    }
}

impl Widget for RowWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        for child in &mut self.children {
            child.mount()?;
        }
        self.layout_children();
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
        self.layout_children();
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

pub struct ColumnWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    gap: f32,
    children: Vec<Element>,
    dirty: bool,
}

impl ColumnWidget {
    pub fn new() -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Center,
            gap: 0.0,
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

    pub fn with_main_axis_alignment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self.dirty = true;
        self
    }

    pub fn with_cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self.dirty = true;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
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

    pub fn layout_children(&mut self) {
        if self.children.is_empty() {
            return;
        }

        // For simplicity, assume each child takes equal height
        let child_count = self.children.len() as f32;
        let total_gap = self.gap * (child_count - 1.0);
        let available_height = self.height - total_gap;
        let child_height = available_height / child_count;

        let mut current_y = self.y;

        // Apply main axis alignment
        match self.main_axis_alignment {
            MainAxisAlignment::Start => {
                current_y = self.y;
            }
            MainAxisAlignment::End => {
                current_y = self.y + self.height - (child_height * child_count + total_gap);
            }
            MainAxisAlignment::Center => {
                current_y = self.y + (self.height - (child_height * child_count + total_gap)) / 2.0;
            }
            MainAxisAlignment::SpaceBetween | 
            MainAxisAlignment::SpaceAround | 
            MainAxisAlignment::SpaceEvenly => {
                current_y = self.y;
            }
        }

        let num_children = self.children.len();
        let gap = self.gap;
        let col_x = self.x;
        let col_width = self.width;
        let cross_alignment = self.cross_axis_alignment;
        
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_x = match cross_alignment {
                CrossAxisAlignment::Start => col_x,
                CrossAxisAlignment::End => col_x + col_width, // Position at right (could subtract child width)
                CrossAxisAlignment::Center => col_x + (col_width / 2.0),
                CrossAxisAlignment::Stretch => col_x,
            };

            // Position the child widget
            Self::position_child_element(child, child_x, current_y, col_width, child_height);
            
            current_y += child_height;
            if i < num_children - 1 {
                current_y += gap;
            }
        }
    }
    
    fn position_child_element(child: &mut Element, x: f32, y: f32, _width: f32, _height: f32) {
        match child {
            Element::Widget(widget) => {
                use crate::widgets::{text::TextWidget, interactive::ButtonWidget, container::BoxWidget};
                
                if let Some(text_widget) = widget.as_any_mut().downcast_mut::<TextWidget>() {
                    text_widget.set_position(x, y);
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    button_widget.set_position(x, y);
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    box_widget.set_position(x, y);
                }
            },
            Element::Container { widget, .. } => {
                use crate::widgets::{text::TextWidget, interactive::ButtonWidget, container::BoxWidget};
                
                if let Some(text_widget) = widget.as_any_mut().downcast_mut::<TextWidget>() {
                    text_widget.set_position(x, y);
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    button_widget.set_position(x, y);
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    box_widget.set_position(x, y);
                }
            },
            Element::Fragment(_) => {
                // Fragments don't have a position
            }
        }
    }
    
    pub fn into_container_element(mut self) -> crate::Element {
        let children = std::mem::take(&mut self.children);
        crate::Element::new_container(Box::new(self), children)
    }
}

impl Widget for ColumnWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        for child in &mut self.children {
            child.mount()?;
        }
        self.layout_children();
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
        self.layout_children();
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

pub struct GridWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rows: usize,
    columns: usize,
    gap: f32,
    children: Vec<Element>,
    dirty: bool,
}

impl GridWidget {
    pub fn new(rows: usize, columns: usize) -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            rows,
            columns,
            gap: 0.0,
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

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
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

    pub fn layout_children(&mut self) {
        if self.children.is_empty() || self.rows == 0 || self.columns == 0 {
            return;
        }

        let horizontal_gaps = (self.columns - 1) as f32 * self.gap;
        let vertical_gaps = (self.rows - 1) as f32 * self.gap;
        
        let cell_width = (self.width - horizontal_gaps) / self.columns as f32;
        let cell_height = (self.height - vertical_gaps) / self.rows as f32;

        for (index, _child) in self.children.iter().enumerate() {
            let row = index / self.columns;
            let col = index % self.columns;
            
            let child_x = self.x + col as f32 * (cell_width + self.gap);
            let child_y = self.y + row as f32 * (cell_height + self.gap);

            // In a real implementation, we would position the child widget here
        }
    }
}

impl Widget for GridWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        for child in &mut self.children {
            child.mount()?;
        }
        self.layout_children();
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
        self.layout_children();
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

// Convenience functions for creating layout widgets
pub fn row() -> RowWidget {
    RowWidget::new()
}

pub fn column() -> ColumnWidget {
    ColumnWidget::new()
}

pub fn grid(rows: usize, columns: usize) -> GridWidget {
    GridWidget::new(rows, columns)
}