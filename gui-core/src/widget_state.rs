use crate::{WidgetId, WidgetError};
use gui_reactive::{Signal, Computed, Effect};
use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::sync::{Arc, RwLock};

pub struct WidgetStateManager {
    states: HashMap<WidgetId, HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    signals: HashMap<(WidgetId, TypeId), Box<dyn Any + Send + Sync>>,
    computed_values: HashMap<(WidgetId, String), Box<dyn Any + Send + Sync>>,
    effects: Vec<Effect>,
}

impl WidgetStateManager {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            signals: HashMap::new(),
            computed_values: HashMap::new(),
            effects: Vec::new(),
        }
    }
    
    pub fn create_state<T>(&mut self, widget_id: WidgetId, initial_value: T) -> StateHandle<T> 
    where 
        T: Clone + Send + Sync + 'static,
    {
        let signal = Signal::new(initial_value.clone());
        let type_id = TypeId::of::<T>();
        
        self.states
            .entry(widget_id)
            .or_insert_with(HashMap::new)
            .insert(type_id, Box::new(initial_value));
            
        self.signals.insert(
            (widget_id, type_id), 
            Box::new(signal.clone())
        );
        
        StateHandle {
            widget_id,
            signal,
            type_id,
        }
    }
    
    pub fn get_state<T>(&self, widget_id: WidgetId) -> Option<StateHandle<T>> 
    where 
        T: Clone + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        
        if let Some(signal_box) = self.signals.get(&(widget_id, type_id)) {
            if let Some(signal) = signal_box.downcast_ref::<Signal<T>>() {
                return Some(StateHandle {
                    widget_id,
                    signal: signal.clone(),
                    type_id,
                });
            }
        }
        None
    }
    
    pub fn create_computed<T, F>(&mut self, widget_id: WidgetId, name: &str, computation: F) -> ComputedHandle<T>
    where 
        T: Clone + Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let computed = Computed::new(computation);
        let key = (widget_id, name.to_string());
        
        self.computed_values.insert(
            key, 
            Box::new(computed.clone())
        );
        
        ComputedHandle {
            widget_id,
            name: name.to_string(),
            computed,
        }
    }
    
    pub fn create_effect<F>(&mut self, widget_id: WidgetId, effect_fn: F) -> EffectHandle
    where 
        F: Fn() + Send + Sync + 'static,
    {
        let effect = Effect::new(effect_fn);
        self.effects.push(effect);
        
        EffectHandle {
            widget_id,
        }
    }
    
    pub fn remove_widget_state(&mut self, widget_id: WidgetId) {
        self.states.remove(&widget_id);
        
        self.signals.retain(|(id, _), _| *id != widget_id);
        self.computed_values.retain(|(id, _), _| *id != widget_id);
        self.effects.retain(|_effect| {
            // Note: In a real implementation, we'd need a way to track which effects belong to which widgets
            true
        });
    }
    
    pub fn update_widget_states(&self, widget_id: WidgetId) -> Result<(), WidgetError> {
        // Trigger updates for all signals and computed values for this widget
        for ((id, _), _signal_box) in &self.signals {
            if *id == widget_id {
                // In a real implementation, we'd trigger signal updates here
            }
        }
        Ok(())
    }
}

impl Default for WidgetStateManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StateHandle<T> 
where 
    T: Clone + Send + Sync + 'static,
{
    widget_id: WidgetId,
    signal: Signal<T>,
    type_id: TypeId,
}

impl<T> StateHandle<T> 
where 
    T: Clone + Send + Sync + 'static,
{
    pub fn get(&self) -> T {
        self.signal.get()
    }
    
    pub fn set(&self, value: T) {
        self.signal.set(value);
    }
    
    pub fn update<F>(&self, updater: F) 
    where 
        F: FnOnce(&mut T),
    {
        self.signal.update(updater);
    }
    
    pub fn widget_id(&self) -> WidgetId {
        self.widget_id
    }
}

impl<T> Clone for StateHandle<T> 
where 
    T: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            widget_id: self.widget_id,
            signal: self.signal.clone(),
            type_id: self.type_id,
        }
    }
}

pub struct ComputedHandle<T> 
where 
    T: Clone + Send + Sync + 'static,
{
    widget_id: WidgetId,
    name: String,
    computed: Computed<T>,
}

impl<T> ComputedHandle<T> 
where 
    T: Clone + Send + Sync + 'static,
{
    pub fn get(&self) -> T {
        self.computed.get()
    }
    
    pub fn widget_id(&self) -> WidgetId {
        self.widget_id
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct EffectHandle {
    widget_id: WidgetId,
}

impl EffectHandle {
    pub fn widget_id(&self) -> WidgetId {
        self.widget_id
    }
}

pub trait StatefulWidget {
    fn get_state_manager(&self) -> &WidgetStateManager;
    fn get_state_manager_mut(&mut self) -> &mut WidgetStateManager;
}

pub struct WidgetStateContext {
    manager: Arc<RwLock<WidgetStateManager>>,
}

impl WidgetStateContext {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(WidgetStateManager::new())),
        }
    }
    
    pub fn with_manager<T, F>(&self, f: F) -> T 
    where 
        F: FnOnce(&WidgetStateManager) -> T,
    {
        let manager = self.manager.read().unwrap();
        f(&*manager)
    }
    
    pub fn with_manager_mut<T, F>(&self, f: F) -> T 
    where 
        F: FnOnce(&mut WidgetStateManager) -> T,
    {
        let mut manager = self.manager.write().unwrap();
        f(&mut *manager)
    }
}

impl Default for WidgetStateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WidgetStateContext {
    fn clone(&self) -> Self {
        Self {
            manager: Arc::clone(&self.manager),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_creation() {
        let mut manager = WidgetStateManager::new();
        let handle = manager.create_state(1, 42i32);
        
        assert_eq!(handle.get(), 42);
        
        handle.set(100);
        assert_eq!(handle.get(), 100);
    }
    
    #[test]
    fn test_state_retrieval() {
        let mut manager = WidgetStateManager::new();
        let _handle1 = manager.create_state(1, 42i32);
        
        let handle2: Option<StateHandle<i32>> = manager.get_state(1);
        assert!(handle2.is_some());
        assert_eq!(handle2.unwrap().get(), 42);
    }
}