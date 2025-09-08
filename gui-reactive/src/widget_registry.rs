use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use crate::Signal;

pub type WidgetId = u64;

#[derive(Clone)]
pub struct ReactiveWidgetRegistry {
    widget_dirty_callbacks: Arc<RwLock<HashMap<WidgetId, Arc<dyn Fn() + Send + Sync>>>>,
}

pub trait WidgetDirtyNotifier: Send + Sync {
    fn mark_widget_dirty(&self, widget_id: WidgetId);
}

impl ReactiveWidgetRegistry {
    pub fn new() -> Self {
        Self {
            widget_dirty_callbacks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_widget_with_notifier(&self, widget_id: WidgetId, notifier: Weak<dyn WidgetDirtyNotifier>) {
        let callback = move || {
            if let Some(strong_notifier) = notifier.upgrade() {
                strong_notifier.mark_widget_dirty(widget_id);
            }
        };
        
        if let Ok(mut callbacks) = self.widget_dirty_callbacks.write() {
            callbacks.insert(widget_id, Arc::new(callback));
        }
    }

    pub fn unregister_widget(&self, widget_id: WidgetId) {
        if let Ok(mut callbacks) = self.widget_dirty_callbacks.write() {
            callbacks.remove(&widget_id);
        }
    }

    pub fn bind_signal_to_widget<T>(&self, signal: &Signal<T>, widget_id: WidgetId)
    where
        T: Clone + Send + Sync + 'static,
    {
        let registry_clone = self.clone();
        signal.subscribe_fn(move |_| {
            registry_clone.notify_widget_dirty(widget_id);
        });
    }

    fn notify_widget_dirty(&self, widget_id: WidgetId) {
        if let Ok(callbacks) = self.widget_dirty_callbacks.read() {
            if let Some(callback) = callbacks.get(&widget_id) {
                callback();
            }
        }
    }
}

impl Default for ReactiveWidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}