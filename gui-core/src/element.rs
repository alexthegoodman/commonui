use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData};
use crate::event::Event;
use crate::widgets::text::TextWidget;
use crate::widgets::container::BoxWidget;
use crate::widgets::interactive::ButtonWidget;

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
}