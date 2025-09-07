use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData};
use crate::event::Event;
use crate::widgets::text::TextWidget;
use crate::widgets::container::BoxWidget;
use crate::widgets::interactive::ButtonWidget;
use crate::widgets::layout::{ColumnWidget, RowWidget};

use vello::Scene;

pub enum Element {
    Widget(Box<dyn Widget>),
    Container {
        widget: Box<dyn Widget>,
        children: Vec<Element>,
    },
    Fragment(Vec<Element>),
}

impl Element {
    pub fn new_widget(widget: Box<dyn Widget>) -> Self {
        Element::Widget(widget)
    }
    
    pub fn new_container(widget: Box<dyn Widget>, children: Vec<Element>) -> Self {
        Element::Container { widget, children }
    }
    
    pub fn new_fragment(children: Vec<Element>) -> Self {
        Element::Fragment(children)
    }
    
    pub fn mount(&mut self) -> Result<(), WidgetError> {
        match self {
            Element::Widget(widget) => widget.mount(),
            Element::Container { widget, children } => {
                widget.mount()?;
                for child in children.iter_mut() {
                    child.mount()?;
                }
                Ok(())
            },
            Element::Fragment(children) => {
                for child in children.iter_mut() {
                    child.mount()?;
                }
                Ok(())
            }
        }
    }
    
    pub fn unmount(&mut self) -> Result<(), WidgetError> {
        match self {
            Element::Widget(widget) => widget.unmount(),
            Element::Container { widget, children } => {
                for child in children.iter_mut() {
                    child.unmount()?;
                }
                widget.unmount()
            },
            Element::Fragment(children) => {
                for child in children.iter_mut() {
                    child.unmount()?;
                }
                Ok(())
            }
        }
    }
    
    pub fn update(&mut self) -> Result<(), WidgetError> {
        match self {
            Element::Widget(widget) => widget.update(),
            Element::Container { widget, children } => {
                widget.update()?;
                
                // Position children for layout widgets
                Element::position_children_for_layout_widget_static(widget.as_ref(), children);
                
                for child in children.iter_mut() {
                    child.update()?;
                }
                Ok(())
            },
            Element::Fragment(children) => {
                for child in children.iter_mut() {
                    child.update()?;
                }
                Ok(())
            }
        }
    }
    
    pub fn handle_event(&mut self, event: &Event) -> EventResult {
        match self {
            Element::Widget(widget) => widget.handle_event(event),
            Element::Container { widget, children } => {
                for child in children.iter_mut() {
                    match child.handle_event(event) {
                        EventResult::Handled => return EventResult::Handled,
                        EventResult::Propagate => continue,
                        EventResult::Ignored => continue,
                    }
                }
                widget.handle_event(event)
            },
            Element::Fragment(children) => {
                for child in children.iter_mut() {
                    match child.handle_event(event) {
                        EventResult::Handled => return EventResult::Handled,
                        EventResult::Propagate => continue,
                        EventResult::Ignored => continue,
                    }
                }
                EventResult::Ignored
            }
        }
    }
    
    pub fn get_widget_by_id(&self, id: WidgetId) -> Option<&dyn Widget> {
        match self {
            Element::Widget(widget) => {
                if widget.get_id() == id {
                    Some(widget.as_ref())
                } else {
                    None
                }
            },
            Element::Container { widget, children } => {
                if widget.get_id() == id {
                    return Some(widget.as_ref());
                }
                for child in children {
                    if let Some(found) = child.get_widget_by_id(id) {
                        return Some(found);
                    }
                }
                None
            },
            Element::Fragment(children) => {
                for child in children {
                    if let Some(found) = child.get_widget_by_id(id) {
                        return Some(found);
                    }
                }
                None
            }
        }
    }
    
    pub fn get_widget_by_id_mut(&mut self, id: WidgetId) -> Option<&mut dyn Widget> {
        match self {
            Element::Widget(widget) => {
                if widget.get_id() == id {
                    Some(widget.as_mut())
                } else {
                    None
                }
            },
            Element::Container { widget, children } => {
                if widget.get_id() == id {
                    return Some(widget.as_mut());
                }
                for child in children {
                    if let Some(found) = child.get_widget_by_id_mut(id) {
                        return Some(found);
                    }
                }
                None
            },
            Element::Fragment(children) => {
                for child in children {
                    if let Some(found) = child.get_widget_by_id_mut(id) {
                        return Some(found);
                    }
                }
                None
            }
        }
    }
    
    pub fn render(&self, scene: &mut Scene, text_renderer: &mut gui_render::primitives::TextRenderer) -> Result<RenderData, WidgetError> {
        match self {
            Element::Widget(widget) => {
                self.render_widget(widget.as_ref(), scene, text_renderer)
            },
            Element::Container { widget, children } => {
                // First render the container widget itself
                let container_render_data = self.render_widget(widget.as_ref(), scene, text_renderer)?;
                
                // Then render all children
                let mut all_dirty_regions = container_render_data.dirty_regions;
                let mut max_z_index = container_render_data.z_index;
                
                for child in children {
                    let child_render_data = child.render(scene, text_renderer)?;
                    all_dirty_regions.extend(child_render_data.dirty_regions);
                    max_z_index = max_z_index.max(child_render_data.z_index);
                }
                
                Ok(RenderData {
                    dirty_regions: all_dirty_regions,
                    z_index: max_z_index,
                })
            },
            Element::Fragment(children) => {
                let mut all_dirty_regions = Vec::new();
                let mut max_z_index = 0;
                
                for child in children {
                    let child_render_data = child.render(scene, text_renderer)?;
                    all_dirty_regions.extend(child_render_data.dirty_regions);
                    max_z_index = max_z_index.max(child_render_data.z_index);
                }
                
                Ok(RenderData {
                    dirty_regions: all_dirty_regions,
                    z_index: max_z_index,
                })
            }
        }
    }
    
    fn render_widget(&self, widget: &dyn Widget, scene: &mut Scene, text_renderer: &mut gui_render::primitives::TextRenderer) -> Result<RenderData, WidgetError> {
        // Get the base render data from the widget
        let render_data = widget.render()?;
        
        // Check the specific widget type and render appropriate primitives
        if let Some(text_widget) = widget.as_any().downcast_ref::<TextWidget>() {
            let text_primitive = text_widget.create_text_primitive();
            text_primitive.draw(scene, text_renderer);
        } else if let Some(box_widget) = widget.as_any().downcast_ref::<BoxWidget>() {
            if let Some(background_rect) = box_widget.create_background_rectangle() {
                background_rect.draw(scene);
            }
        } else if let Some(button_widget) = widget.as_any().downcast_ref::<ButtonWidget>() {
            let background_rect = button_widget.create_background_rectangle();
            background_rect.draw(scene);
            if let Some(text_primitive) = button_widget.create_text_primitive() {
                text_primitive.draw(scene, text_renderer);
            }
        }
        
        Ok(render_data)
    }
    
    fn position_children_for_layout_widget_static(widget: &dyn Widget, children: &mut Vec<Element>) {
        if let Some(column_widget) = widget.as_any().downcast_ref::<ColumnWidget>() {
            Element::position_children_for_column_static(column_widget, children);
        } else if let Some(row_widget) = widget.as_any().downcast_ref::<RowWidget>() {
            Element::position_children_for_row_static(row_widget, children);
        } else if let Some(box_widget) = widget.as_any().downcast_ref::<BoxWidget>() {
            Element::position_children_for_box_static(box_widget, children);
        }
    }
    
    fn position_children_for_box_static(box_widget: &BoxWidget, children: &mut Vec<Element>) {
        if children.is_empty() {
            return;
        }

        let (content_x, content_y, content_width, content_height) = box_widget.get_content_area();
        // println!("Box content area: x={}, y={}, w={}, h={}", content_x, content_y, content_width, content_height);
        
        // Position all children at the content area position and let layout widgets handle their own children
        for child in children.iter_mut() {
            Element::position_child_element_static(child, content_x, content_y, content_width, content_height);
            
            // If the child is a container with a layout widget, apply its layout logic
            if let Element::Container { widget, children: child_children } = child {
                Element::position_children_for_layout_widget_static(widget.as_ref(), child_children);
            }
        }
    }
    
    fn position_children_for_column_static(column_widget: &ColumnWidget, children: &mut Vec<Element>) {
        if children.is_empty() {
            return;
        }

        let child_count = children.len() as f32;
        let gap = column_widget.get_gap();
        let total_gap = gap * (child_count - 1.0);
        let (col_x, col_y) = column_widget.get_position();
        let (col_width, col_height) = column_widget.get_size();
        // println!("Column positioned at x={}, y={}, w={}, h={}", col_x, col_y, col_width, col_height);
        let available_height = col_height - total_gap;
        let child_height = available_height / child_count;

        let mut current_y = col_y;

        // Apply main axis alignment
        match column_widget.get_main_axis_alignment() {
            crate::widgets::layout::MainAxisAlignment::Start => {
                current_y = col_y;
            }
            crate::widgets::layout::MainAxisAlignment::End => {
                current_y = col_y + col_height - (child_height * child_count + total_gap);
            }
            crate::widgets::layout::MainAxisAlignment::Center => {
                current_y = col_y + (col_height - (child_height * child_count + total_gap)) / 2.0;
            }
            crate::widgets::layout::MainAxisAlignment::SpaceBetween | 
            crate::widgets::layout::MainAxisAlignment::SpaceAround | 
            crate::widgets::layout::MainAxisAlignment::SpaceEvenly => {
                current_y = col_y;
            }
        }

        let cross_alignment = column_widget.get_cross_axis_alignment();
        let num_children = children.len();
        
        for (i, child) in children.iter_mut().enumerate() {
            // Position the child widget with proper alignment
            Element::position_child_element_for_alignment(child, col_x, current_y, col_width, child_height, cross_alignment);
            
            current_y += child_height;
            if i < (num_children - 1) {
                current_y += gap;
            }
        }
    }
    
    fn position_children_for_row_static(row_widget: &RowWidget, children: &mut Vec<Element>) {
        if children.is_empty() {
            return;
        }

        let child_count = children.len() as f32;
        let gap = row_widget.get_gap();
        let total_gap = gap * (child_count - 1.0);
        let (row_x, row_y) = row_widget.get_position();
        let (row_width, row_height) = row_widget.get_size();
        let available_width = row_width - total_gap;
        let child_width = available_width / child_count;

        let mut current_x = row_x;

        // Apply main axis alignment
        match row_widget.get_main_axis_alignment() {
            crate::widgets::layout::MainAxisAlignment::Start => {
                current_x = row_x;
            }
            crate::widgets::layout::MainAxisAlignment::End => {
                current_x = row_x + row_width - (child_width * child_count + total_gap);
            }
            crate::widgets::layout::MainAxisAlignment::Center => {
                current_x = row_x + (row_width - (child_width * child_count + total_gap)) / 2.0;
            }
            crate::widgets::layout::MainAxisAlignment::SpaceBetween | 
            crate::widgets::layout::MainAxisAlignment::SpaceAround | 
            crate::widgets::layout::MainAxisAlignment::SpaceEvenly => {
                current_x = row_x;
            }
        }

        let cross_alignment = row_widget.get_cross_axis_alignment();
        let num_children = children.len();
        
        for (i, child) in children.iter_mut().enumerate() {
            let child_y = match cross_alignment {
                crate::widgets::layout::CrossAxisAlignment::Start => row_y,
                crate::widgets::layout::CrossAxisAlignment::End => row_y + row_height,
                crate::widgets::layout::CrossAxisAlignment::Center => row_y + (row_height / 2.0),
                crate::widgets::layout::CrossAxisAlignment::Stretch => row_y,
            };

            // Position the child widget
            Element::position_child_element_static(child, current_x, child_y, child_width, row_height);
            
            current_x += child_width;
            if i < (num_children - 1) {
                current_x += gap;
            }
        }
    }
    
    fn position_child_element_static(child: &mut Element, x: f32, y: f32, _width: f32, _height: f32) {
        // println!("Positioning child at x={}, y={}", x, y);
        match child {
            Element::Widget(widget) => {
                use crate::widgets::{text::TextWidget, interactive::ButtonWidget, container::BoxWidget, layout::ColumnWidget};
                
                if let Some(text_widget) = widget.as_any_mut().downcast_mut::<TextWidget>() {
                    // println!("  Positioning text widget at x={}, y={}", x, y);
                    text_widget.set_position(x, y);
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    // println!("  Positioning button widget at x={}, y={}", x, y);
                    button_widget.set_position(x, y);
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    // println!("  Positioning box widget at x={}, y={}", x, y);
                    box_widget.set_position(x, y);
                } else if let Some(column_widget) = widget.as_any_mut().downcast_mut::<ColumnWidget>() {
                    // println!("  Positioning column widget at x={}, y={}", x, y);
                    column_widget.set_position(x, y);
                }
            },
            Element::Container { widget, .. } => {
                use crate::widgets::{text::TextWidget, interactive::ButtonWidget, container::BoxWidget, layout::ColumnWidget};
                
                if let Some(text_widget) = widget.as_any_mut().downcast_mut::<TextWidget>() {
                    // println!("  Positioning text widget (container) at x={}, y={}", x, y);
                    text_widget.set_position(x, y);
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    // println!("  Positioning button widget (container) at x={}, y={}", x, y);
                    button_widget.set_position(x, y);
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    // println!("  Positioning box widget (container) at x={}, y={}", x, y);
                    box_widget.set_position(x, y);
                } else if let Some(column_widget) = widget.as_any_mut().downcast_mut::<ColumnWidget>() {
                    // println!("  Positioning column widget (container) at x={}, y={}", x, y);
                    column_widget.set_position(x, y);
                }
            },
            Element::Fragment(_) => {
                // Fragments don't have a position
            }
        }
    }
    
    fn position_child_element_for_alignment(child: &mut Element, x: f32, y: f32, container_width: f32, _height: f32, cross_alignment: crate::widgets::layout::CrossAxisAlignment) {
        match child {
            Element::Widget(widget) => {
                use crate::widgets::{text::TextWidget, interactive::ButtonWidget, container::BoxWidget, layout::ColumnWidget};
                
                if let Some(text_widget) = widget.as_any_mut().downcast_mut::<TextWidget>() {
                    let final_x = match cross_alignment {
                        crate::widgets::layout::CrossAxisAlignment::Start => x,
                        crate::widgets::layout::CrossAxisAlignment::End => {
                            let (text_width, _) = text_widget.measure_text();
                            x + container_width - text_width
                        },
                        crate::widgets::layout::CrossAxisAlignment::Center => {
                            let (text_width, _) = text_widget.measure_text();
                            x + (container_width - text_width) / 2.0
                        },
                        crate::widgets::layout::CrossAxisAlignment::Stretch => x,
                    };
                    text_widget.set_position(final_x, y);
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    button_widget.set_position(x, y);
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    box_widget.set_position(x, y);
                } else if let Some(column_widget) = widget.as_any_mut().downcast_mut::<ColumnWidget>() {
                    column_widget.set_position(x, y);
                }
            },
            Element::Container { widget, .. } => {
                use crate::widgets::{text::TextWidget, interactive::ButtonWidget, container::BoxWidget, layout::ColumnWidget};
                
                if let Some(text_widget) = widget.as_any_mut().downcast_mut::<TextWidget>() {
                    let final_x = match cross_alignment {
                        crate::widgets::layout::CrossAxisAlignment::Start => x,
                        crate::widgets::layout::CrossAxisAlignment::End => {
                            let (text_width, _) = text_widget.measure_text();
                            x + container_width - text_width
                        },
                        crate::widgets::layout::CrossAxisAlignment::Center => {
                            let (text_width, _) = text_widget.measure_text();
                            x + (container_width - text_width) / 2.0
                        },
                        crate::widgets::layout::CrossAxisAlignment::Stretch => x,
                    };
                    text_widget.set_position(final_x, y);
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    button_widget.set_position(x, y);
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    box_widget.set_position(x, y);
                } else if let Some(column_widget) = widget.as_any_mut().downcast_mut::<ColumnWidget>() {
                    column_widget.set_position(x, y);
                }
            },
            Element::Fragment(_) => {
                // Fragments don't have a position
            }
        }
    }
}