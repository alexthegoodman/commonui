use std::sync::{Arc, RwLock, Mutex};
use std::collections::VecDeque;
use crate::signal::{Signal, SignalId};

type BatchUpdate = Box<dyn FnOnce() + Send>;

pub struct BatchManager {
    pending_updates: Arc<Mutex<VecDeque<BatchUpdate>>>,
    batching_enabled: Arc<RwLock<bool>>,
}

impl BatchManager {
    pub fn new() -> Self {
        Self {
            pending_updates: Arc::new(Mutex::new(VecDeque::new())),
            batching_enabled: Arc::new(RwLock::new(false)),
        }
    }

    pub fn start_batch(&self) {
        if let Ok(mut enabled) = self.batching_enabled.write() {
            *enabled = true;
        }
    }

    pub fn flush_batch(&self) {
        if let Ok(mut enabled) = self.batching_enabled.write() {
            *enabled = false;
        }

        if let Ok(mut updates) = self.pending_updates.lock() {
            while let Some(update) = updates.pop_front() {
                update();
            }
        }
    }

    pub fn queue_update<F>(&self, update: F) -> bool 
    where 
        F: FnOnce() + Send + 'static,
    {
        if let Ok(enabled) = self.batching_enabled.read() {
            if *enabled {
                if let Ok(mut updates) = self.pending_updates.lock() {
                    updates.push_back(Box::new(update));
                    return true;
                }
            }
        }
        false
    }

    pub fn is_batching(&self) -> bool {
        self.batching_enabled.read().map(|enabled| *enabled).unwrap_or(false)
    }
}

impl Default for BatchManager {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL_BATCH_MANAGER: std::sync::OnceLock<BatchManager> = std::sync::OnceLock::new();

pub fn global_batch_manager() -> &'static BatchManager {
    GLOBAL_BATCH_MANAGER.get_or_init(BatchManager::new)
}

pub fn batch_updates<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let manager = global_batch_manager();
    manager.start_batch();
    let result = f();
    manager.flush_batch();
    result
}

pub struct BatchedSignal<T> {
    inner: Signal<T>,
}

impl<T> BatchedSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(initial_value: T) -> Self {
        Self {
            inner: Signal::new(initial_value),
        }
    }

    pub fn from_signal(signal: Signal<T>) -> Self {
        Self { inner: signal }
    }

    pub fn get(&self) -> T {
        self.inner.get()
    }

    pub fn set(&self, new_value: T) {
        let signal = self.inner.clone();
        let value = new_value.clone();
        let update = move || signal.set(value);

        if !global_batch_manager().queue_update(update) {
            self.inner.set(new_value);
        }
    }

    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T) + Send + 'static,
    {
        if global_batch_manager().is_batching() {
            let signal = self.inner.clone();
            let update = move || signal.update(updater);
            let _ = global_batch_manager().queue_update(update);
        } else {
            self.inner.update(updater);
        }
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.inner.with(f)
    }

    pub fn id(&self) -> SignalId {
        self.inner.id()
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<T> {
        self.inner.subscribe()
    }

    pub fn subscribe_fn<F>(&self, callback: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.inner.subscribe_fn(callback)
    }

    pub fn into_signal(self) -> Signal<T> {
        self.inner
    }
}

impl<T> Clone for BatchedSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> From<Signal<T>> for BatchedSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn from(signal: Signal<T>) -> Self {
        Self::from_signal(signal)
    }
}

impl<T> From<BatchedSignal<T>> for Signal<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn from(batched: BatchedSignal<T>) -> Self {
        batched.into_signal()
    }
}