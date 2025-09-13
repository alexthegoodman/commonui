use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, DirtyRegion, WidgetUpdateContext};
use crate::event::Event;
use crate::element::Element;
use gui_reactive::Signal;
use gui_render::primitives::{Rectangle, Text};
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use vello::peniko::Color;
use super::container::{Background, BoxWidget, Padding, container};
use super::layout::{column, row};
use super::text::text;
use super::interactive::{input, InputWidget};
use super::dropdown::{dropdown, DropdownWidget, DropdownOption};

static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(5000);

#[derive(Clone, Debug)]
pub enum PropertyValue {
    Text(String),
    Number(f32),
    Color(Color),
    Boolean(bool),
    Select(String), // Selected value from dropdown
}

#[derive(Clone, Debug)]
pub struct PropertyDefinition {
    pub key: String,
    pub label: String,
    pub value: PropertyValue,
    pub property_type: PropertyType,
}

#[derive(Clone, Debug)]
pub enum PropertyType {
    TextInput,
    NumberInput,
    ColorPicker,
    Checkbox,
    Dropdown(Vec<DropdownOption>),
}

impl PropertyDefinition {
    pub fn text(key: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value: PropertyValue::Text(value.into()),
            property_type: PropertyType::TextInput,
        }
    }
    
    pub fn number(key: impl Into<String>, label: impl Into<String>, value: f32) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value: PropertyValue::Number(value),
            property_type: PropertyType::NumberInput,
        }
    }
    
    pub fn dropdown(key: impl Into<String>, label: impl Into<String>, options: Vec<DropdownOption>, selected: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value: PropertyValue::Select(selected.into()),
            property_type: PropertyType::Dropdown(options),
        }
    }
    
    pub fn color(key: impl Into<String>, label: impl Into<String>, value: Color) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value: PropertyValue::Color(value),
            property_type: PropertyType::ColorPicker,
        }
    }
    
    pub fn boolean(key: impl Into<String>, label: impl Into<String>, value: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value: PropertyValue::Boolean(value),
            property_type: PropertyType::Checkbox,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PropertyGroup {
    pub title: String,
    pub properties: Vec<PropertyDefinition>,
    pub expanded: bool,
}

impl PropertyGroup {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            properties: Vec::new(),
            expanded: true,
        }
    }
    
    pub fn with_property(mut self, property: PropertyDefinition) -> Self {
        self.properties.push(property);
        self
    }
    
    pub fn with_properties(mut self, properties: Vec<PropertyDefinition>) -> Self {
        self.properties.extend(properties);
        self
    }
    
    pub fn collapsed(mut self) -> Self {
        self.expanded = false;
        self
    }
}

pub struct PropertyInspectorWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    groups: Vec<PropertyGroup>,
    background: Background,
    header_background: Background,
    border_color: Color,
    text_color: Color,
    header_text_color: Color,
    font_size: f32,
    header_font_size: f32,
    padding: Padding,
    row_height: f32,
    header_height: f32,
    on_property_change: Option<Box<dyn Fn(&str, &PropertyValue) + Send + Sync>>,
    pub dirty: bool,
    pub children: Vec<Element>,
}

impl PropertyInspectorWidget {
    pub fn new() -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 600.0,
            groups: Vec::new(),
            background: Background::Color(Color::rgba8(45, 45, 50, 255)),
            header_background: Background::Color(Color::rgba8(35, 35, 40, 255)),
            border_color: Color::rgba8(60, 60, 65, 255),
            text_color: Color::rgba8(220, 220, 220, 255),
            header_text_color: Color::rgba8(255, 255, 255, 255),
            font_size: 12.0,
            header_font_size: 14.0,
            padding: Padding::all(8.0),
            row_height: 32.0,
            header_height: 28.0,
            on_property_change: None,
            dirty: true,
            children: Vec::new(),
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn with_groups(mut self, groups: Vec<PropertyGroup>) -> Self {
        self.groups = groups;
        self.dirty = true;
        self
    }

    pub fn add_group(mut self, group: PropertyGroup) -> Self {
        self.groups.push(group);
        self.dirty = true;
        self
    }

    pub fn on_property_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &PropertyValue) + Send + Sync + 'static,
    {
        self.on_property_change = Some(Box::new(callback));
        self
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        if self.x != x || self.y != y {
            self.x = x;
            self.y = y;
            self.dirty = true;
        }
    }

    pub fn get_children(&self) -> &Vec<Element> {
        &self.children
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }

    pub fn get_position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn get_size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    pub fn get_padding(&self) -> &Padding {
        &self.padding
    }

    pub fn get_header_height(&self) -> f32 {
        self.header_height
    }

    pub fn get_row_height(&self) -> f32 {
        self.row_height
    }

    pub fn update_property(&mut self, key: &str, value: PropertyValue) {
        for group in &mut self.groups {
            for property in &mut group.properties {
                if property.key == key {
                    property.value = value.clone();
                    if let Some(ref callback) = self.on_property_change {
                        callback(key, &value);
                    }
                    self.dirty = true;
                    return;
                }
            }
        }
    }

    pub fn rebuild_ui(&mut self) {
        self.children.clear();
        let mut current_y = self.y + self.padding.top;

        for group in &self.groups {
            // Create group header
            let header_container = container()
                .with_size(self.width - self.padding.left - self.padding.right, self.header_height)
                .with_background(self.header_background.clone())
                .with_child(
                    text(&group.title)
                        .with_color(self.header_text_color)
                        .with_font_size(self.header_font_size)
                        .into_text_element()
                );

            self.children.push(header_container.into_container_element());
            current_y += self.header_height + 4.0;

            if group.expanded {
                // Create property rows
                for property in &group.properties {
                    let row_element = self.create_property_row(property, current_y);
                    self.children.push(row_element);
                    current_y += self.row_height + 4.0;
                }
            }

            current_y += 8.0; // Group spacing
        }
    }

    fn create_property_row(&self, property: &PropertyDefinition, y: f32) -> Element {
        let label_width = (self.width - self.padding.left - self.padding.right) * 0.4;
        let input_width = (self.width - self.padding.left - self.padding.right) * 0.6 - 8.0;

        let label_element = text(&property.label)
            .with_color(self.text_color)
            .with_font_size(self.font_size)
            .into_text_element();

        let input_element = match &property.property_type {
            PropertyType::TextInput => {
                let text_value = match &property.value {
                    PropertyValue::Text(s) => s.clone(),
                    _ => String::new(),
                };
                
                input()
                    .with_size(input_width, self.row_height - 4.0)
                    .with_text(text_value)
                    .into_input_element()
            },
            PropertyType::NumberInput => {
                let number_value = match &property.value {
                    PropertyValue::Number(n) => n.to_string(),
                    _ => "0".to_string(),
                };
                
                input()
                    .with_size(input_width, self.row_height - 4.0)
                    .with_text(number_value)
                    .into_input_element()
            },
            PropertyType::Dropdown(options) => {
                let selected_value = match &property.value {
                    PropertyValue::Select(s) => s.clone(),
                    _ => String::new(),
                };
                
                dropdown()
                    .with_size(input_width, self.row_height - 4.0)
                    .with_options(options.clone())
                    .with_selected_value(selected_value)
                    .into_dropdown_element()
            },
            PropertyType::ColorPicker => {
                // For now, represent color as hex string
                let color_value = match &property.value {
                    PropertyValue::Color(c) => format!("#{:02x}{:02x}{:02x}", 
                        (c.r as f32 * 255.0) as u8, 
                        (c.g as f32 * 255.0) as u8, 
                        (c.b as f32 * 255.0) as u8),
                    _ => "#000000".to_string(),
                };
                
                input()
                    .with_size(input_width, self.row_height - 4.0)
                    .with_text(color_value)
                    .into_input_element()
            },
            PropertyType::Checkbox => {
                // For now, represent as text "true" or "false"
                let bool_value = match &property.value {
                    PropertyValue::Boolean(b) => b.to_string(),
                    _ => "false".to_string(),
                };
                
                input()
                    .with_size(input_width, self.row_height - 4.0)
                    .with_text(bool_value)
                    .into_input_element()
            },
        };

        row()
            .with_size(self.width - self.padding.left - self.padding.right, self.row_height)
            .with_child(label_element)
            .with_child(input_element)
            .into_container_element()
    }

    pub fn create_background_rectangle(&self) -> Rectangle {
        Rectangle::new_with_brush(self.x, self.y, self.width, self.height, self.background.to_brush())
            .with_stroke_width(1.0)
    }
}

impl Widget for PropertyInspectorWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        self.rebuild_ui();
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
            self.rebuild_ui();
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
            z_index: 2,
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

// Convenience function for creating property inspectors
pub fn property_inspector() -> PropertyInspectorWidget {
    PropertyInspectorWidget::new()
}

impl PropertyInspectorWidget {
    pub fn into_property_inspector_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }

    /// Convert the PropertyInspectorWidget to a Container element that can manage children
    /// This allows the property inspector to be used like other layout widgets (row, column, etc.)
    pub fn into_container_element(mut self) -> Element {
        let children = std::mem::take(&mut self.children);
        Element::new_container(Box::new(self), children)
    }
}

// Extension traits to convert widgets to elements
trait IntoElement<T> {
    fn into_dropdown_element(self) -> Element;
    fn into_input_element(self) -> Element;
    fn into_text_element(self) -> Element;
    fn into_row_element(self) -> Element;
}

impl IntoElement<DropdownWidget> for DropdownWidget {
    fn into_dropdown_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_input_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_text_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_row_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
}

impl IntoElement<InputWidget> for InputWidget {
    fn into_input_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_dropdown_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_text_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_row_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
}

use super::text::TextWidget;
use super::layout::RowWidget;

impl IntoElement<TextWidget> for TextWidget {
    fn into_text_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_dropdown_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_input_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_row_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
}

// impl IntoElement<RowWidget> for RowWidget {
//     fn into_row_element(self) -> Element {
//         Element::new_container(Box::new(self), Vec::new())
//     }
    
//     fn into_dropdown_element(self) -> Element {
//         Element::new_container(Box::new(self), Vec::new())
//     }
    
//     fn into_input_element(self) -> Element {
//         Element::new_container(Box::new(self), Vec::new())
//     }
    
//     fn into_text_element(self) -> Element {
//         Element::new_container(Box::new(self), Vec::new())
//     }
// }

impl IntoElement<RowWidget> for RowWidget {
    fn into_text_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_dropdown_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_input_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
    
    fn into_row_element(self) -> Element {
        Element::new_widget(Box::new(self))
    }
}