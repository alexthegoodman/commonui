use std::sync::{Arc, RwLock, Weak};
use tokio::sync::broadcast;

pub type SignalId = usize;

pub struct Signal<T> {
    id: SignalId,
    value: Arc<RwLock<T>>,
    sender: broadcast::Sender<T>,
    subscribers: Arc<RwLock<Vec<Weak<dyn Fn(&T) + Send + Sync>>>>,
}

impl<T> Signal<T> 
where 
    T: Clone + Send + Sync + 'static,
{
    pub fn new(initial_value: T) -> Self {
        let (sender, _) = broadcast::channel(1024);
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        
        Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            value: Arc::new(RwLock::new(initial_value)),
            sender,
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    pub fn set(&self, new_value: T) {
        {
            let mut value = self.value.write().unwrap();
            *value = new_value.clone();
        }
        
        // Notify subscribers
        let _ = self.sender.send(new_value.clone());
        self.notify_subscribers(&new_value);
    }

    pub fn update<F>(&self, updater: F) 
    where 
        F: FnOnce(&mut T),
    {
        let new_value = {
            let mut value = self.value.write().unwrap();
            updater(&mut *value);
            value.clone()
        };
        
        // Notify subscribers
        let _ = self.sender.send(new_value.clone());
        self.notify_subscribers(&new_value);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }

    pub fn with<F, R>(&self, f: F) -> R 
    where 
        F: FnOnce(&T) -> R,
    {
        let value = self.value.read().unwrap();
        f(&*value)
    }

    pub fn id(&self) -> SignalId {
        self.id
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

impl<T> Clone for Signal<T> 
where 
    T: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: self.value.clone(),
            sender: self.sender.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}