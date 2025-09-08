use crate::{Element, Widget, WidgetId, WidgetError, EventResult, WidgetUpdateContext};
use crate::event::Event;
use std::collections::HashMap;
use gui_reactive::Signal;

pub struct WidgetManager {
    root: Option<Element>,
    mounted_widgets: HashMap<WidgetId, bool>,
    widget_registry: HashMap<WidgetId, String>,
    dirty_widgets: Signal<Vec<WidgetId>>,
}

struct WidgetManagerUpdateContext<'a> {
    dirty_widgets: &'a Signal<Vec<WidgetId>>,
}

impl WidgetManager {
    pub fn new() -> Self {
        Self {
            root: None,
            mounted_widgets: HashMap::new(),
            widget_registry: HashMap::new(),
            dirty_widgets: Signal::new(Vec::new()),
        }
    }
    
    pub fn set_root(&mut self, mut element: Element) -> Result<(), WidgetError> {
        if let Some(mut old_root) = self.root.take() {
            self.unmount_element(&mut old_root)?;
        }
        
        self.mount_element(&mut element)?;
        self.root = Some(element);
        Ok(())
    }
    
    pub fn mount_element(&mut self, element: &mut Element) -> Result<(), WidgetError> {
        match element {
            Element::Widget(widget) => {
                let widget_id = widget.get_id();
                if !*self.mounted_widgets.get(&widget_id).unwrap_or(&false) {
                    widget.mount()?;
                    self.mounted_widgets.insert(widget_id, true);
                    self.widget_registry.insert(widget_id, format!("Widget_{}", widget_id));
                }
                Ok(())
            },
            Element::Container { widget, children } => {
                let widget_id = widget.get_id();
                if !*self.mounted_widgets.get(&widget_id).unwrap_or(&false) {
                    widget.mount()?;
                    self.mounted_widgets.insert(widget_id, true);
                    self.widget_registry.insert(widget_id, format!("Widget_{}", widget_id));
                }
                for child in children.iter_mut() {
                    self.mount_element(child)?;
                }
                Ok(())
            },
            Element::Fragment(children) => {
                for child in children.iter_mut() {
                    self.mount_element(child)?;
                }
                Ok(())
            }
        }
    }
    
    pub fn unmount_element(&mut self, element: &mut Element) -> Result<(), WidgetError> {
        match element {
            Element::Widget(widget) => {
                let widget_id = widget.get_id();
                if *self.mounted_widgets.get(&widget_id).unwrap_or(&false) {
                    widget.unmount()?;
                    self.mounted_widgets.insert(widget_id, false);
                    self.widget_registry.remove(&widget_id);
                }
                Ok(())
            },
            Element::Container { widget, children } => {
                for child in children.iter_mut() {
                    self.unmount_element(child)?;
                }
                let widget_id = widget.get_id();
                if *self.mounted_widgets.get(&widget_id).unwrap_or(&false) {
                    widget.unmount()?;
                    self.mounted_widgets.insert(widget_id, false);
                    self.widget_registry.remove(&widget_id);
                }
                Ok(())
            },
            Element::Fragment(children) => {
                for child in children.iter_mut() {
                    self.unmount_element(child)?;
                }
                Ok(())
            }
        }
    }
    
    pub fn update_all(&mut self) -> Result<(), WidgetError> {
        if self.root.is_some() {
            // Create a temporary context that implements WidgetUpdateContext
            let context = WidgetManagerUpdateContext {
                dirty_widgets: &self.dirty_widgets,
            };
            self.root.as_mut().unwrap().update(&context)
        } else {
            Ok(())
        }
    }
    
    pub fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Some(ref mut root) = self.root {
            root.handle_event(event)
        } else {
            EventResult::Ignored
        }
    }
    
    pub fn get_widget(&self, id: WidgetId) -> Option<&dyn Widget> {
        if let Some(ref root) = self.root {
            root.get_widget_by_id(id)
        } else {
            None
        }
    }
    
    pub fn get_widget_mut(&mut self, id: WidgetId) -> Option<&mut dyn Widget> {
        if let Some(ref mut root) = self.root {
            root.get_widget_by_id_mut(id)
        } else {
            None
        }
    }
    
    pub fn is_widget_mounted(&self, id: WidgetId) -> bool {
        *self.mounted_widgets.get(&id).unwrap_or(&false)
    }
    
    pub fn mark_widget_dirty(&self, id: WidgetId) {
        let mut dirty_list = self.dirty_widgets.get();
        if !dirty_list.contains(&id) {
            dirty_list.push(id);
            self.dirty_widgets.set(dirty_list);
        }
    }
    
    pub fn get_dirty_widgets(&self) -> Vec<WidgetId> {
        self.dirty_widgets.get()
    }
    
    pub fn clear_dirty_widgets(&self) {
        self.dirty_widgets.set(Vec::new());
    }

    pub fn root(&self) -> Option<&Element> {
        self.root.as_ref()
    }
}

impl<'a> WidgetUpdateContext for WidgetManagerUpdateContext<'a> {
    fn mark_dirty(&self, widget_id: WidgetId) {
        let mut dirty_list = self.dirty_widgets.get();
        if !dirty_list.contains(&widget_id) {
            dirty_list.push(widget_id);
            self.dirty_widgets.set(dirty_list);
        }
    }
}

impl WidgetUpdateContext for WidgetManager {
    fn mark_dirty(&self, widget_id: WidgetId) {
        self.mark_widget_dirty(widget_id);
    }
}

impl Default for WidgetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderData, DirtyRegion};
    use std::any::Any;
    
    struct TestWidget {
        id: WidgetId,
        mounted: bool,
    }
    
    impl TestWidget {
        fn new(id: WidgetId) -> Self {
            Self { id, mounted: false }
        }
    }
    
    impl Widget for TestWidget {
        fn mount(&mut self) -> Result<(), WidgetError> {
            self.mounted = true;
            Ok(())
        }
        
        fn unmount(&mut self) -> Result<(), WidgetError> {
            self.mounted = false;
            Ok(())
        }
        
        fn update(&mut self, _ctx: &dyn WidgetUpdateContext) -> Result<(), WidgetError> {
            Ok(())
        }
        
        fn render(&self) -> Result<RenderData, WidgetError> {
            Ok(RenderData {
                dirty_regions: vec![],
                z_index: 0,
            })
        }
        
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
        fn get_id(&self) -> WidgetId { self.id }
    }
    
    #[test]
    fn test_widget_mounting() {
        let mut manager = WidgetManager::new();
        let widget = Box::new(TestWidget::new(1));
        let element = Element::new_widget(widget);
        
        manager.set_root(element).unwrap();
        assert!(manager.is_widget_mounted(1));
    }
}