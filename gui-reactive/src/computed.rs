use std::sync::{Arc, RwLock, Weak};
use tokio::sync::broadcast;
use crate::signal::{Signal, SignalId};

pub struct Computed<T> {
    id: SignalId,
    value: Arc<RwLock<Option<T>>>,
    sender: broadcast::Sender<T>,
    subscribers: Arc<RwLock<Vec<Weak<dyn Fn(&T) + Send + Sync>>>>,
    computation: Arc<dyn Fn() -> T + Send + Sync>,
    dependencies: Vec<SignalId>,
    is_dirty: Arc<RwLock<bool>>,
}

impl<T> Computed<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new<F>(computation: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let (sender, _) = broadcast::channel(1024);
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

        Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            value: Arc::new(RwLock::new(None)),
            sender,
            subscribers: Arc::new(RwLock::new(Vec::new())),
            computation: Arc::new(computation),
            dependencies: Vec::new(),
            is_dirty: Arc::new(RwLock::new(true)),
        }
    }

    pub fn from_signal<F>(signal: &Signal<impl Clone + Send + Sync + 'static>, computation: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let computed = Self::new(computation);
        
        // Subscribe to the signal for updates
        let computed_clone = computed.clone();
        signal.subscribe_fn(move |_| {
            computed_clone.mark_dirty();
        });

        computed
    }

    pub fn from_signals<F, U>(signals: &[&Signal<U>], computation: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
        U: Clone + Send + Sync + 'static,
    {
        let computed = Self::new(computation);
        
        // Subscribe to all signals for updates
        for signal in signals {
            let computed_clone = computed.clone();
            signal.subscribe_fn(move |_| {
                computed_clone.mark_dirty();
            });
        }

        computed
    }

    pub fn get(&self) -> T {
        let is_dirty = *self.is_dirty.read().unwrap();
        
        if is_dirty || self.value.read().unwrap().is_none() {
            self.recompute();
        }

        self.value.read().unwrap().as_ref().unwrap().clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }

    pub fn subscribe_fn<F>(&self, callback: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let callback_arc = Arc::new(callback);
        let weak_ref = Arc::downgrade(&(callback_arc.clone() as Arc<dyn Fn(&T) + Send + Sync>));
        
        if let Ok(mut subscribers) = self.subscribers.write() {
            subscribers.push(weak_ref);
        }
        
        // Call immediately with current value
        let current_value = self.get();
        callback_arc(&current_value);
    }

    pub fn id(&self) -> SignalId {
        self.id
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let value = self.get();
        f(&value)
    }

    fn recompute(&self) {
        let new_value = (self.computation)();
        
        {
            let mut value = self.value.write().unwrap();
            *value = Some(new_value.clone());
        }
        
        {
            let mut is_dirty = self.is_dirty.write().unwrap();
            *is_dirty = false;
        }

        // Notify subscribers
        let _ = self.sender.send(new_value.clone());
        self.notify_subscribers(&new_value);
    }

    fn mark_dirty(&self) {
        let mut is_dirty = self.is_dirty.write().unwrap();
        if !*is_dirty {
            *is_dirty = true;
            
            // Trigger recomputation on next access
            // For now, we could also eagerly recompute here if desired
        }
    }

    fn notify_subscribers(&self, value: &T) {
        if let Ok(mut subscribers) = self.subscribers.write() {
            // Clean up dead weak references and call live ones
            subscribers.retain(|weak_callback| {
                if let Some(callback) = weak_callback.upgrade() {
                    callback(value);
                    true
                } else {
                    false
                }
            });
        }
    }
}

impl<T> Clone for Computed<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: self.value.clone(),
            sender: self.sender.clone(),
            subscribers: self.subscribers.clone(),
            computation: self.computation.clone(),
            dependencies: self.dependencies.clone(),
            is_dirty: self.is_dirty.clone(),
        }
    }
}