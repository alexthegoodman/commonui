use crate::{Widget, WidgetId, EventResult, WidgetError};
use crate::event::Event;

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
}