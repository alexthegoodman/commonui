use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, WidgetUpdateContext};
use crate::event::Event;
use crate::widgets::text::TextWidget;
use crate::widgets::container::BoxWidget;
use crate::widgets::interactive::{ButtonWidget, InputWidget, SliderWidget};
use crate::widgets::layout::{ColumnWidget, RowWidget};
use crate::widgets::canvas::CanvasWidget;
use crate::widgets::property_inspector::PropertyInspectorWidget;

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
    
    pub fn update(&mut self, ctx: &mut dyn WidgetUpdateContext) -> Result<(), WidgetError> {
        match self {
            Element::Widget(widget) => {
                // println!("widget update");
                widget.update(ctx)?;

                // dont update children here, they are updated recursively in the widget.update() function
                
                Ok(())
            },
            Element::Container { widget, children } => {
                widget.update(ctx)?;
                
                // Position children for layout widgets
                if let Some(column_widget) = widget.as_any().downcast_ref::<ColumnWidget>() {
                    let (col_x, col_y) = column_widget.get_position();
                    let (col_width, col_height) = column_widget.get_size();
                    // println!("column {:?} {:?}", col_x, col_y);
                    Element::position_children_for_column_with_coords(column_widget, children, col_x, col_y, col_width, col_height);
                } else if let Some(row_widget) = widget.as_any().downcast_ref::<RowWidget>() {
                    let (row_x, row_y) = row_widget.get_position();
                    let (row_width, row_height) = row_widget.get_size();
                    Element::position_children_for_row_with_coords(row_widget, children, row_x, row_y, row_width, row_height);
                } else if let Some(box_widget) = widget.as_any().downcast_ref::<BoxWidget>() {
                    Element::position_children_for_box_static(box_widget, children);
                } else if let Some(inspector_widget) = widget.as_any_mut().downcast_mut::<PropertyInspectorWidget>() {
                    Element::position_children_for_property_inspector(inspector_widget);
                }
                
                for child in children.iter_mut() {
                    child.update(ctx)?;
                }
                Ok(())
            },
            Element::Fragment(children) => {
                for child in children.iter_mut() {
                    child.update(ctx)?;
                }
                Ok(())
            }
        }
    }
    
    pub fn handle_event(&mut self, event: &Event) -> EventResult {
        match self {
            Element::Widget(widget) => widget.handle_event(event),
            Element::Container { widget, children } => {
                // Special handling for containers with display control
                use crate::widgets::container::BoxWidget;
                if let Some(box_widget) = widget.as_any().downcast_ref::<BoxWidget>() {
                    if !box_widget.is_visible() {
                        // Container is hidden, don't process children
                        return EventResult::Ignored;
                    }
                }
                
                // For normal containers, process children first, then container
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
    
    pub fn execute_direct_render_functions(&self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, view_width: u32, view_height: u32) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Element::Widget(widget) => {
                if let Some(canvas_widget) = widget.as_any().downcast_ref::<CanvasWidget>() {
                    if canvas_widget.has_direct_render_func() {
                        canvas_widget.execute_direct_render(device, queue, view, view_width, view_height)?;
                    }
                }
            },
            Element::Container { widget, children } => {
                if let Some(canvas_widget) = widget.as_any().downcast_ref::<CanvasWidget>() {
                    if canvas_widget.has_direct_render_func() {
                        canvas_widget.execute_direct_render(device, queue, view, view_width, view_height)?;
                    }
                }
                for child in children {
                    child.execute_direct_render_functions(device, queue, view, view_width, view_height)?;
                }
            },
            Element::Fragment(children) => {
                for child in children {
                    child.execute_direct_render_functions(device, queue, view, view_width, view_height)?;
                }
            }
        }
        Ok(())
    }

    pub fn render(&self, scene: &mut Scene, text_renderer: &mut gui_render::primitives::TextRenderer, device: Option<&wgpu::Device>, queue: Option<&wgpu::Queue>) -> Result<RenderData, WidgetError> {
        match self {
            Element::Widget(widget) => {
                // First render the widget itself
                let widget_render_data = self.render_widget(widget.as_ref(), scene, text_renderer, device, queue)?;
                
                // Check if this is a layout widget that has children and render them too
                if let Some(column_widget) = widget.as_any().downcast_ref::<ColumnWidget>() {
                    let mut all_dirty_regions = widget_render_data.dirty_regions;
                    let mut max_z_index = widget_render_data.z_index;
                    
                    for child in column_widget.get_children() {
                        let child_render_data = child.render(scene, text_renderer, device, queue)?;
                        all_dirty_regions.extend(child_render_data.dirty_regions);
                        max_z_index = max_z_index.max(child_render_data.z_index);
                    }
                    
                    Ok(RenderData {
                        dirty_regions: all_dirty_regions,
                        z_index: max_z_index,
                    })
                } else if let Some(row_widget) = widget.as_any().downcast_ref::<RowWidget>() {
                    let mut all_dirty_regions = widget_render_data.dirty_regions;
                    let mut max_z_index = widget_render_data.z_index;
                    
                    for child in row_widget.get_children() {
                        let child_render_data = child.render(scene, text_renderer, device, queue)?;
                        all_dirty_regions.extend(child_render_data.dirty_regions);
                        max_z_index = max_z_index.max(child_render_data.z_index);
                    }
                    
                    Ok(RenderData {
                        dirty_regions: all_dirty_regions,
                        z_index: max_z_index,
                    })
                } else if let Some(box_widget) = widget.as_any().downcast_ref::<BoxWidget>() {
                    let mut all_dirty_regions = widget_render_data.dirty_regions;
                    let mut max_z_index = widget_render_data.z_index;
                    
                    for child in box_widget.get_children() {
                        let child_render_data = child.render(scene, text_renderer, device, queue)?;
                        all_dirty_regions.extend(child_render_data.dirty_regions);
                        max_z_index = max_z_index.max(child_render_data.z_index);
                    }
                    
                    Ok(RenderData {
                        dirty_regions: all_dirty_regions,
                        z_index: max_z_index,
                    })
                } else if let Some(inspector_widget) = widget.as_any().downcast_ref::<PropertyInspectorWidget>() {
                    let mut all_dirty_regions = widget_render_data.dirty_regions;
                    let mut max_z_index = widget_render_data.z_index;
                    
                    for child in inspector_widget.get_children() {
                        let child_render_data = child.render(scene, text_renderer, device, queue)?;
                        all_dirty_regions.extend(child_render_data.dirty_regions);
                        max_z_index = max_z_index.max(child_render_data.z_index);
                    }
                    
                    Ok(RenderData {
                        dirty_regions: all_dirty_regions,
                        z_index: max_z_index,
                    })
                } else {
                    Ok(widget_render_data)
                }
            },
            Element::Container { widget, children } => {
                // First render the container widget itself
                let container_render_data = self.render_widget(widget.as_ref(), scene, text_renderer, device, queue)?;
                
                // Then render all children
                let mut all_dirty_regions = container_render_data.dirty_regions;
                let mut max_z_index = container_render_data.z_index;

                if widget.needs_render() {                
                    for child in children {
                        // let mut needs_render = true;
                        // match child {
                        //     Element::Widget(widget) => {
                        //         needs_render = widget.needs_render();
                        //     },
                        //     Element::Container { widget, children } => {
                        //         needs_render = widget.needs_render();
                        //     },
                        //     Element::Fragment(children) => {
                        //         needs_render = true;
                        //     }
                        // }

                        let child_render_data = child.render(scene, text_renderer, device, queue)?;
                        all_dirty_regions.extend(child_render_data.dirty_regions);
                        max_z_index = max_z_index.max(child_render_data.z_index);
                    }
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
                    let child_render_data = child.render(scene, text_renderer, device, queue)?;
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
    
    fn render_widget(&self, widget: &dyn Widget, scene: &mut Scene, text_renderer: &mut gui_render::primitives::TextRenderer, device: Option<&wgpu::Device>, queue: Option<&wgpu::Queue>) -> Result<RenderData, WidgetError> {
        // Get the base render data from the widget
        let needs_render = widget.needs_render();

        if !needs_render {
            return Ok(RenderData {
                dirty_regions: vec![],
                z_index: 0,
            });
        }

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
            // let (x, y) = button_widget.get_position();
            // let (w, h) = button_widget.get_size();
            // println!("Rendering button at x={}, y={}, w={}, h={}", x, y, w, h);
            let background_rect = button_widget.create_background_rectangle();
            // println!("Button background color: {:?}", button_widget.get_current_color());
            background_rect.draw(scene);
            if let Some(text_primitive) = button_widget.create_text_primitive() {
                text_primitive.draw(scene, text_renderer);
            }
        } else if let Some(input_widget) = widget.as_any().downcast_ref::<InputWidget>() {
            // Render shadow first if it exists
            if let Some(shadow) = input_widget.create_shadow() {
                shadow.draw(scene);
            }
            // Render background rectangle
            let background_rect = input_widget.create_background_rectangle();
            background_rect.draw(scene);
            // Render text (either current text or placeholder)
            if let Some(text_primitive) = input_widget.create_text_primitive() {
                text_primitive.draw(scene, text_renderer);
            }
        } else if let Some(slider_widget) = widget.as_any().downcast_ref::<SliderWidget>() {
            // Render shadow first if it exists
            if let Some(shadow) = slider_widget.create_shadow() {
                shadow.draw(scene);
            }
            // Render track
            let track_rect = slider_widget.create_track_rectangle();
            track_rect.draw(scene);
            // Render fill
            let fill_rect = slider_widget.create_fill_rectangle();
            fill_rect.draw(scene);
        } else if let Some(canvas_widget) = widget.as_any().downcast_ref::<CanvasWidget>() {
            // Render Canvas widget with custom render function
            if let (Some(device), Some(queue)) = (device, queue) {
                canvas_widget.render_to_scene(scene, device, queue)?;
            }
            // Note: Direct render functions are handled separately in the App layer
        } else if let Some(inspector_widget) = widget.as_any().downcast_ref::<PropertyInspectorWidget>() {
            // Render PropertyInspectorWidget background
            let background_rect = inspector_widget.create_background_rectangle();
            background_rect.draw(scene);
        }
        
        Ok(render_data)
    }
    
    
    fn position_children_for_box_static(box_widget: &BoxWidget, children: &mut Vec<Element>) {
        if children.is_empty() {
            return;
        }

        let (content_x, content_y, content_width, content_height) = box_widget.get_content_area();
        // println!("Box content area: x={}, y={}, w={}, h={}", content_x, content_y, content_width, content_height);
        
        // Position children in a column layout to prevent overlapping
        // Calculate height per child based on available content height
        let child_count = children.len() as f32;
        let child_height = content_height / child_count;
        let mut current_y = content_y;
        
        for child in children.iter_mut() {
            Element::position_child_element_static(child, content_x, current_y, content_width, child_height);
            current_y += child_height;
        }
        
        // Then, let layout widgets handle their own children (after they've been positioned)
        for child in children.iter_mut() {
            if let Element::Widget(widget) = child {
                if let Some(inspector_widget) = widget.as_any_mut().downcast_mut::<PropertyInspectorWidget>() {
                    Element::position_children_for_property_inspector(inspector_widget);
                }
            } 
            if let Element::Container { widget, children: child_children } = child {
                // Get the widget's current position and size
                if let Some(column_widget) = widget.as_any().downcast_ref::<ColumnWidget>() {
                    let (col_x, col_y) = column_widget.get_position();
                    let (col_width, col_height) = column_widget.get_size();
                    // println!("Position column {:?} {:?}", col_x, col_y);
                    // sets positions for all children
                    Element::position_children_for_column_with_coords(column_widget, child_children, col_x, col_y, col_width, col_height);
                } else if let Some(row_widget) = widget.as_any().downcast_ref::<RowWidget>() {
                    let (row_x, row_y) = row_widget.get_position();
                    let (row_width, row_height) = row_widget.get_size();
                    Element::position_children_for_row_with_coords(row_widget, child_children, row_x, row_y, row_width, row_height);
                } else if let Some(inspector_widget) = widget.as_any_mut().downcast_mut::<PropertyInspectorWidget>() {
                    Element::position_children_for_property_inspector(inspector_widget);
                }
            }
        }
    }
    
    fn position_children_for_column_with_coords(column_widget: &ColumnWidget, children: &mut Vec<Element>, col_x: f32, col_y: f32, col_width: f32, col_height: f32) {
        if children.is_empty() {
            return;
        }

        // Count only normal flow children (skip absolutely positioned ones)
        let normal_children_count = children.iter()
            .filter(|child| !Element::is_absolutely_positioned(child))
            .count();
            
        if normal_children_count == 0 {
            return;
        }

        let child_count = normal_children_count as f32;
        let gap = column_widget.get_gap();
        let total_gap = gap * (child_count - 1.0);
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
        let mut normal_child_index = 0;
        
        for child in children.iter_mut() {
            // Skip absolutely positioned children from normal layout flow
            if Element::is_absolutely_positioned(child) {
                continue;
            }
            
            // Position the child widget with proper alignment
            // println!("position_child_element_for_alignment {:?} {:?}", col_x, current_y);
            Element::position_child_element_for_alignment(child, col_x, current_y, col_width, child_height, cross_alignment);
            
            current_y += child_height;
            if normal_child_index < (normal_children_count - 1) {
                current_y += gap;
            }
            
            normal_child_index += 1;
        }
    }
    
    fn is_absolutely_positioned(child: &Element) -> bool {
        use gui_layout::Position;
        use crate::widgets::container::BoxWidget;
        
        match child {
            Element::Widget(widget) => {
                if let Some(box_widget) = widget.as_any().downcast_ref::<BoxWidget>() {
                    box_widget.get_position_type() == Position::Absolute
                } else {
                    false
                }
            },
            Element::Container { widget, .. } => {
                if let Some(box_widget) = widget.as_any().downcast_ref::<BoxWidget>() {
                    box_widget.get_position_type() == Position::Absolute
                } else {
                    false
                }
            },
            Element::Fragment(_) => false,
        }
    }

    fn position_children_for_row_with_coords(row_widget: &RowWidget, children: &mut Vec<Element>, row_x: f32, row_y: f32, row_width: f32, row_height: f32) {
        if children.is_empty() {
            return;
        }

        // Count only normal flow children (skip absolutely positioned ones)
        let normal_children_count = children.iter()
            .filter(|child| !Element::is_absolutely_positioned(child))
            .count();
            
        if normal_children_count == 0 {
            return;
        }

        let child_count = normal_children_count as f32;
        let gap = row_widget.get_gap();
        let total_gap = gap * (child_count - 1.0);
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
        let mut normal_child_index = 0;
        
        for child in children.iter_mut() {
            // Skip absolutely positioned children from normal layout flow
            if Element::is_absolutely_positioned(child) {
                continue;
            }
            
            let child_y = match cross_alignment {
                crate::widgets::layout::CrossAxisAlignment::Start => row_y,
                crate::widgets::layout::CrossAxisAlignment::End => row_y + row_height,
                crate::widgets::layout::CrossAxisAlignment::Center => row_y + (row_height / 2.0),
                crate::widgets::layout::CrossAxisAlignment::Stretch => row_y,
            };

            // Position the child widget in normal flow
            Element::position_child_element_static(child, current_x, child_y, child_width, row_height);
            
            current_x += child_width;
            if normal_child_index < (normal_children_count - 1) {
                current_x += gap;
            }
            
            normal_child_index += 1;
        }
    }
    
    fn position_children_for_property_inspector(inspector_widget: &mut PropertyInspectorWidget) {
        let (inspector_x, inspector_y) = inspector_widget.get_position();
        let (inspector_width, _inspector_height) = inspector_widget.get_size();
        let padding = inspector_widget.get_padding();
        
        // PropertyInspectorWidget manages its own internal layout
        // We just need to position the children within the inspector's content area
        let content_x = inspector_x + padding.left;
        let mut current_y = inspector_y + padding.top;
        
        let header_height = inspector_widget.get_header_height();
        let row_height = inspector_widget.get_row_height();
        let content_width = inspector_width - padding.left - padding.right;

        let children = &mut inspector_widget.children;

        println!("position child prop insp almost....");

        if children.is_empty() {
            return;
        }
        
        // Position children based on PropertyInspector's internal structure
        // Headers and property rows are arranged vertically
        println!("position child prop insp");
        for child in children.iter_mut() {
            Element::position_child_element_static(child, content_x, current_y, content_width, row_height);
            
            // Check if this child is a header or a property row and advance accordingly
            match child {
                Element::Widget(_) | Element::Container { .. } => {
                    // For now, treat all children as rows with standard spacing
                    current_y += row_height + 4.0;
                },
                Element::Fragment(_) => {
                    // Fragments might contain multiple elements
                    current_y += row_height + 4.0;
                }
            }
        }
    }
    
    fn position_child_element_static(child: &mut Element, x: f32, y: f32, _width: f32, _height: f32) {
        // println!("Positioning child at x={}, y={}", x, y);
        match child {
            Element::Widget(widget) => {
                use crate::widgets::{text::TextWidget, interactive::{ButtonWidget, InputWidget, SliderWidget}, container::BoxWidget, layout::{ColumnWidget, RowWidget}, canvas::CanvasWidget};
                
                if let Some(text_widget) = widget.as_any_mut().downcast_mut::<TextWidget>() {
                    // println!("  Positioning text widget at x={}, y={}", x, y);
                    text_widget.set_position(x, y);
                    text_widget.dirty = true;
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    // println!("  Positioning button widget at x={}, y={}", x, y);
                    button_widget.set_position(x, y);
                    button_widget.dirty = true;
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    // println!("  Positioning box widget at x={}, y={}", x, y);
                    box_widget.set_position(x, y);
                    box_widget.dirty = true;
                } else if let Some(column_widget) = widget.as_any_mut().downcast_mut::<ColumnWidget>() {
                    // println!("  Positioning column widget at x={}, y={}", x, y);
                    column_widget.set_position(x, y);
                    column_widget.dirty = true;
                } else if let Some(row_widget) = widget.as_any_mut().downcast_mut::<RowWidget>() {
                    row_widget.set_position(x, y);
                    row_widget.dirty = true;
                } else if let Some(input_widget) = widget.as_any_mut().downcast_mut::<InputWidget>() {
                    input_widget.set_position(x, y);
                    input_widget.dirty = true;
                } else if let Some(slider_widget) = widget.as_any_mut().downcast_mut::<SliderWidget>() {
                    slider_widget.set_position(x, y);
                    slider_widget.dirty = true;
                } else if let Some(canvas_widget) = widget.as_any_mut().downcast_mut::<CanvasWidget>() {
                    canvas_widget.set_position(x, y);
                    canvas_widget.dirty = true;
                }
            },
            Element::Container { widget, .. } => {
                use crate::widgets::{text::TextWidget, interactive::{ButtonWidget, InputWidget, SliderWidget}, container::BoxWidget, layout::{ColumnWidget, RowWidget}, canvas::CanvasWidget};
                
                if let Some(text_widget) = widget.as_any_mut().downcast_mut::<TextWidget>() {
                    // println!("  Positioning text widget (container) at x={}, y={}", x, y);
                    text_widget.set_position(x, y);
                    text_widget.dirty = true;
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    // println!("  Positioning button widget (container) at x={}, y={}", x, y);
                    button_widget.set_position(x, y);
                    button_widget.dirty = true;
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    // println!("  Positioning box widget (container) at x={}, y={}", x, y);
                    box_widget.set_position(x, y);
                    box_widget.dirty = true;
                } else if let Some(column_widget) = widget.as_any_mut().downcast_mut::<ColumnWidget>() {
                    // println!("  Positioning column widget (container) at x={}, y={}", x, y);
                    column_widget.set_position(x, y);
                    column_widget.dirty = true;
                } else if let Some(row_widget) = widget.as_any_mut().downcast_mut::<RowWidget>() {
                    row_widget.set_position(x, y);
                    row_widget.dirty = true;
                } else if let Some(canvas_widget) = widget.as_any_mut().downcast_mut::<CanvasWidget>() {
                    canvas_widget.set_position(x, y);
                    canvas_widget.dirty = true;
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
                use crate::widgets::{text::TextWidget, interactive::{ButtonWidget, InputWidget, SliderWidget}, container::BoxWidget, layout::{ColumnWidget, RowWidget}, canvas::CanvasWidget};
                
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
                    // println!("text y a {:?}", y);
                    text_widget.set_position(final_x, y);
                    text_widget.dirty = true;
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    button_widget.set_position(x, y);
                    button_widget.dirty = true;
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    box_widget.set_position(x, y);
                    box_widget.dirty = true;
                } else if let Some(column_widget) = widget.as_any_mut().downcast_mut::<ColumnWidget>() {
                    column_widget.set_position(x, y);
                    column_widget.dirty = true;
                } else if let Some(row_widget) = widget.as_any_mut().downcast_mut::<RowWidget>() {
                    row_widget.set_position(x, y);
                    row_widget.dirty = true;
                } else if let Some(input_widget) = widget.as_any_mut().downcast_mut::<InputWidget>() {
                    input_widget.set_position(x, y);
                    input_widget.dirty = true;
                } else if let Some(slider_widget) = widget.as_any_mut().downcast_mut::<SliderWidget>() {
                    slider_widget.set_position(x, y);
                    slider_widget.dirty = true;
                } else if let Some(canvas_widget) = widget.as_any_mut().downcast_mut::<CanvasWidget>() {
                    canvas_widget.set_position(x, y);
                    canvas_widget.dirty = true;
                }
            },
            Element::Container { widget, .. } => {
                use crate::widgets::{text::TextWidget, interactive::{ButtonWidget, InputWidget, SliderWidget}, container::BoxWidget, layout::{ColumnWidget, RowWidget}, canvas::CanvasWidget};
                
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
                    // println!("text y b {:?}", y);
                    text_widget.set_position(final_x, y);
                    text_widget.dirty = true;
                } else if let Some(button_widget) = widget.as_any_mut().downcast_mut::<ButtonWidget>() {
                    button_widget.set_position(x, y);
                    button_widget.dirty = true;
                } else if let Some(box_widget) = widget.as_any_mut().downcast_mut::<BoxWidget>() {
                    box_widget.set_position(x, y);
                    box_widget.dirty = true;
                } else if let Some(column_widget) = widget.as_any_mut().downcast_mut::<ColumnWidget>() {
                    column_widget.set_position(x, y);
                    column_widget.dirty = true;
                } else if let Some(row_widget) = widget.as_any_mut().downcast_mut::<RowWidget>() {
                    row_widget.set_position(x, y);
                    row_widget.dirty = true;
                } else if let Some(input_widget) = widget.as_any_mut().downcast_mut::<InputWidget>() {
                    input_widget.set_position(x, y);
                    input_widget.dirty = true;
                } else if let Some(slider_widget) = widget.as_any_mut().downcast_mut::<SliderWidget>() {
                    slider_widget.set_position(x, y);
                    slider_widget.dirty = true;
                } else if let Some(canvas_widget) = widget.as_any_mut().downcast_mut::<CanvasWidget>() {
                    canvas_widget.set_position(x, y);
                    canvas_widget.dirty = true;
                }
            },
            Element::Fragment(_) => {
                // Fragments don't have a position
            }
        }
    }

    /// Creates a combined shared encoder render function from all Canvas widgets in the element tree
    pub fn create_combined_shared_encoder_render_func(&self) -> Option<impl Fn(&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder, &[vello::ExternalResource]) -> Result<(), vello::Error> + Send + Sync + 'static> {
        use crate::widgets::canvas::CanvasWidget;
        use std::sync::Arc;
        use vello::ExternalResource;
        
        let mut render_functions = Vec::new();
        self.collect_canvas_shared_encoder_funcs(&mut render_functions);
        
        if render_functions.is_empty() {
            return None;
        }
        
        Some(move |device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, external_resources: &[ExternalResource]| -> Result<(), vello::Error> {
            for render_func in &render_functions {
                render_func(device, queue, encoder, external_resources)?;
            }
            Ok(())
        })
    }
    
    fn collect_canvas_shared_encoder_funcs(&self, functions: &mut Vec<Box<dyn Fn(&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder, &[vello::ExternalResource]) -> Result<(), vello::Error> + Send + Sync>>) {
        use crate::widgets::canvas::CanvasWidget;
        
        match self {
            Element::Widget(widget) => {
                if let Some(canvas_widget) = widget.as_any().downcast_ref::<CanvasWidget>() {
                    if let Some(func) = canvas_widget.create_shared_encoder_render_func() {
                        functions.push(Box::new(func));
                    }
                }
            },
            Element::Container { widget, children } => {
                if let Some(canvas_widget) = widget.as_any().downcast_ref::<CanvasWidget>() {
                    if let Some(func) = canvas_widget.create_shared_encoder_render_func() {
                        functions.push(Box::new(func));
                    }
                }
                for child in children {
                    child.collect_canvas_shared_encoder_funcs(functions);
                }
            },
            Element::Fragment(children) => {
                for child in children {
                    child.collect_canvas_shared_encoder_funcs(functions);
                }
            }
        }
    }
}