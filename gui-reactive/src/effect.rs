use std::sync::Arc;
use tokio::task::JoinHandle;
use crate::signal::{Signal, SignalId};
use crate::computed::Computed;

pub type EffectId = usize;

pub struct Effect {
    id: EffectId,
    cleanup: Option<Arc<dyn Fn() + Send + Sync>>,
    task_handle: Option<JoinHandle<()>>,
    dependencies: Vec<SignalId>,
}

impl Effect {
    pub fn new<F>(effect_fn: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

        let effect = Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            cleanup: None,
            task_handle: None,
            dependencies: Vec::new(),
        };

        // Run the effect immediately
        effect_fn();

        effect
    }

    pub fn from_signal<F, T>(signal: &Signal<T>, effect_fn: F) -> Self
    where
        F: Fn(&T) + Send + Sync + 'static,
        T: Clone + Send + Sync + 'static,
    {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

        let effect = Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            cleanup: None,
            task_handle: None,
            dependencies: vec![signal.id()],
        };

        // Subscribe to signal changes
        signal.subscribe_fn(effect_fn);

        effect
    }

    pub fn from_signals<F, T>(signals: &[&Signal<T>], effect_fn: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
        T: Clone + Send + Sync + 'static,
    {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

        let dependencies: Vec<SignalId> = signals.iter().map(|s| s.id()).collect();

        let effect = Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            cleanup: None,
            task_handle: None,
            dependencies,
        };

        // Subscribe to all signal changes
        let effect_arc = Arc::new(effect_fn);
        for signal in signals {
            let effect_clone = effect_arc.clone();
            signal.subscribe_fn(move |_| {
                effect_clone();
            });
        }

        effect
    }

    pub fn from_computed<F, T>(computed: &Computed<T>, effect_fn: F) -> Self
    where
        F: Fn(&T) + Send + Sync + 'static,
        T: Clone + Send + Sync + 'static,
    {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

        let effect = Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            cleanup: None,
            task_handle: None,
            dependencies: vec![computed.id()],
        };

        // Subscribe to computed value changes
        computed.subscribe_fn(effect_fn);

        effect
    }

    pub fn with_cleanup<F, C>(mut self, cleanup_fn: C) -> Self
    where
        C: Fn() + Send + Sync + 'static,
    {
        self.cleanup = Some(Arc::new(cleanup_fn));
        self
    }

    pub fn run_async<F, Fut>(mut self, async_effect: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(async_effect());
        self.task_handle = Some(handle);
        self
    }

    pub fn id(&self) -> EffectId {
        self.id
    }

    pub fn dispose(mut self) {
        // Run cleanup if provided
        if let Some(cleanup) = &self.cleanup {
            cleanup();
        }

        // Cancel async task if running
        if let Some(handle) = &self.task_handle {
            handle.abort();
        }

        // Clear dependencies
        self.dependencies.clear();
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        // Cleanup on drop
        if let Some(cleanup) = &self.cleanup {
            cleanup();
        }

        // Cancel async task if running
        if let Some(handle) = &self.task_handle {
            handle.abort();
        }
    }
}

pub struct EffectRunner {
    effects: Vec<Effect>,
}

impl EffectRunner {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    pub fn remove_effect(&mut self, effect_id: EffectId) {
        self.effects.retain(|e| e.id() != effect_id);
    }

    pub fn clear_all(&mut self) {
        self.effects.clear();
    }

    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }
}

impl Default for EffectRunner {
    fn default() -> Self {
        Self::new()
    }
}