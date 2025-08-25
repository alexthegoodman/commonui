use std::sync::{Arc, RwLock, Mutex};
use std::collections::{HashMap, HashSet};
use std::any::Any;
use tokio::sync::oneshot;
use std::time::{Duration, Instant};

use crate::{
    Signal, Computed, Effect, 
    signal::SignalId,
    batch::BatchManager,
    frame_sync::FrameScheduler,
    threading::ThreadManager
};

pub type RuntimeId = usize;

#[derive(Debug, Clone)]
pub enum RuntimeState {
    Initializing,
    Running,
    Pausing,
    Paused,
    ShuttingDown,
    Shutdown,
}

pub struct SignalRegistry {
    signals: HashMap<SignalId, Box<dyn Any + Send + Sync>>,
    signal_metadata: HashMap<SignalId, SignalMetadata>,
    next_cleanup: Instant,
    cleanup_interval: Duration,
}

#[derive(Debug, Clone)]
struct SignalMetadata {
    created_at: Instant,
    last_accessed: Instant,
    subscriber_count: usize,
    type_name: &'static str,
}

impl SignalRegistry {
    pub fn new() -> Self {
        Self {
            signals: HashMap::new(),
            signal_metadata: HashMap::new(),
            next_cleanup: Instant::now() + Duration::from_secs(30),
            cleanup_interval: Duration::from_secs(30),
        }
    }

    pub fn register_signal<T>(&mut self, signal: &Signal<T>, type_name: &'static str) 
    where 
        T: Clone + Send + Sync + 'static,
    {
        let now = Instant::now();
        let metadata = SignalMetadata {
            created_at: now,
            last_accessed: now,
            subscriber_count: 0,
            type_name,
        };

        self.signals.insert(signal.id(), Box::new(signal.clone()));
        self.signal_metadata.insert(signal.id(), metadata);
    }

    pub fn unregister_signal(&mut self, signal_id: SignalId) {
        self.signals.remove(&signal_id);
        self.signal_metadata.remove(&signal_id);
    }

    pub fn get_signal<T>(&mut self, signal_id: SignalId) -> Option<Signal<T>>
    where
        T: Clone + Send + Sync + 'static,
    {
        if let Some(signal_box) = self.signals.get(&signal_id) {
            if let Some(signal) = signal_box.downcast_ref::<Signal<T>>() {
                // Update last accessed time
                if let Some(metadata) = self.signal_metadata.get_mut(&signal_id) {
                    metadata.last_accessed = Instant::now();
                }
                return Some(signal.clone());
            }
        }
        None
    }

    pub fn update_subscriber_count(&mut self, signal_id: SignalId, count: usize) {
        if let Some(metadata) = self.signal_metadata.get_mut(&signal_id) {
            metadata.subscriber_count = count;
        }
    }

    pub fn cleanup_unused_signals(&mut self) -> usize {
        if Instant::now() < self.next_cleanup {
            return 0;
        }

        let cutoff = Instant::now() - Duration::from_secs(300); // 5 minutes
        let mut removed = 0;

        let to_remove: Vec<SignalId> = self.signal_metadata
            .iter()
            .filter_map(|(id, metadata)| {
                if metadata.subscriber_count == 0 && metadata.last_accessed < cutoff {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        for signal_id in to_remove {
            self.unregister_signal(signal_id);
            removed += 1;
        }

        self.next_cleanup = Instant::now() + self.cleanup_interval;
        removed
    }

    pub fn get_signal_count(&self) -> usize {
        self.signals.len()
    }

    pub fn get_signals_by_type(&self, type_name: &str) -> Vec<SignalId> {
        self.signal_metadata
            .iter()
            .filter_map(|(id, metadata)| {
                if metadata.type_name == type_name {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
}

pub struct ComputedRegistry {
    computeds: HashMap<usize, Box<dyn Any + Send + Sync>>,
    dependencies: HashMap<usize, HashSet<SignalId>>,
}

impl ComputedRegistry {
    pub fn new() -> Self {
        Self {
            computeds: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn register_computed<T>(&mut self, computed: &Computed<T>, dependencies: Vec<SignalId>)
    where
        T: Clone + Send + Sync + 'static,
    {
        let computed_id = computed as *const Computed<T> as usize;
        self.computeds.insert(computed_id, Box::new(computed.clone()));
        self.dependencies.insert(computed_id, dependencies.into_iter().collect());
    }

    pub fn unregister_computed<T>(&mut self, computed: &Computed<T>) {
        let computed_id = computed as *const Computed<T> as usize;
        self.computeds.remove(&computed_id);
        self.dependencies.remove(&computed_id);
    }

    pub fn get_dependents(&self, signal_id: SignalId) -> Vec<usize> {
        self.dependencies
            .iter()
            .filter_map(|(computed_id, deps)| {
                if deps.contains(&signal_id) {
                    Some(*computed_id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_computed_count(&self) -> usize {
        self.computeds.len()
    }
}

pub struct EffectRegistry {
    effect_ids: HashSet<usize>,
    active_effects: HashSet<usize>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self {
            effect_ids: HashSet::new(),
            active_effects: HashSet::new(),
        }
    }

    pub fn register_effect(&mut self, effect: &Effect) {
        let effect_id = effect as *const Effect as usize;
        self.effect_ids.insert(effect_id);
        self.active_effects.insert(effect_id);
    }

    pub fn unregister_effect(&mut self, effect: &Effect) {
        let effect_id = effect as *const Effect as usize;
        self.effect_ids.remove(&effect_id);
        self.active_effects.remove(&effect_id);
    }

    pub fn dispose_all_effects(&mut self) {
        // Clear all effects - they should handle their own cleanup in Drop
        self.effect_ids.clear();
        self.active_effects.clear();
    }

    pub fn get_active_effect_count(&self) -> usize {
        self.active_effects.len()
    }

    pub fn get_total_effect_count(&self) -> usize {
        self.effect_ids.len()
    }
}

pub struct ReactiveRuntime {
    id: RuntimeId,
    state: Arc<RwLock<RuntimeState>>,
    signal_registry: Arc<Mutex<SignalRegistry>>,
    computed_registry: Arc<Mutex<ComputedRegistry>>,
    effect_registry: Arc<Mutex<EffectRegistry>>,
    batch_manager: Arc<BatchManager>,
    frame_scheduler: Arc<FrameScheduler>,
    thread_manager: Arc<ThreadManager>,
    shutdown_sender: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    created_at: Instant,
}

impl ReactiveRuntime {
    pub fn new() -> (Self, oneshot::Receiver<()>) {
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let runtime = Self {
            id,
            state: Arc::new(RwLock::new(RuntimeState::Initializing)),
            signal_registry: Arc::new(Mutex::new(SignalRegistry::new())),
            computed_registry: Arc::new(Mutex::new(ComputedRegistry::new())),
            effect_registry: Arc::new(Mutex::new(EffectRegistry::new())),
            batch_manager: Arc::new(BatchManager::new()),
            frame_scheduler: Arc::new(FrameScheduler::new(60)),
            thread_manager: Arc::new(ThreadManager::new().0),
            shutdown_sender: Arc::new(Mutex::new(Some(shutdown_sender))),
            created_at: Instant::now(),
        };

        (runtime, shutdown_receiver)
    }

    pub fn initialize(&self) -> Result<(), String> {
        if let Ok(mut state) = self.state.write() {
            match *state {
                RuntimeState::Initializing => {
                    *state = RuntimeState::Running;
                    Ok(())
                }
                _ => Err(format!("Cannot initialize runtime in state: {:?}", *state))
            }
        } else {
            Err("Failed to acquire state lock for initialization".to_string())
        }
    }

    pub fn register_signal<T>(&self, signal: &Signal<T>) 
    where 
        T: Clone + Send + Sync + 'static,
    {
        if let Ok(mut registry) = self.signal_registry.lock() {
            registry.register_signal(signal, std::any::type_name::<T>());
        }
    }

    pub fn unregister_signal(&self, signal_id: SignalId) {
        if let Ok(mut registry) = self.signal_registry.lock() {
            registry.unregister_signal(signal_id);
        }
    }

    pub fn register_computed<T>(&self, computed: &Computed<T>, dependencies: Vec<SignalId>)
    where
        T: Clone + Send + Sync + 'static,
    {
        if let Ok(mut registry) = self.computed_registry.lock() {
            registry.register_computed(computed, dependencies);
        }
    }

    pub fn unregister_computed<T>(&self, computed: &Computed<T>)
    where
        T: Clone + Send + Sync + 'static,
    {
        if let Ok(mut registry) = self.computed_registry.lock() {
            registry.unregister_computed(computed);
        }
    }

    pub fn register_effect(&self, effect: &Effect) {
        if let Ok(mut registry) = self.effect_registry.lock() {
            registry.register_effect(effect);
        }
    }

    pub fn unregister_effect(&self, effect: &Effect) {
        if let Ok(mut registry) = self.effect_registry.lock() {
            registry.unregister_effect(effect);
        }
    }

    pub fn cleanup_unused_resources(&self) -> CleanupStats {
        let signals_removed = if let Ok(mut registry) = self.signal_registry.lock() {
            registry.cleanup_unused_signals()
        } else {
            0
        };

        CleanupStats {
            signals_removed,
            computed_removed: 0, // Computed values are cleaned up via weak references
            effects_disposed: 0, // Effects are cleaned up via Drop trait
        }
    }

    pub fn get_runtime_stats(&self) -> RuntimeStats {
        let signal_count = self.signal_registry.lock()
            .map(|registry| registry.get_signal_count())
            .unwrap_or(0);

        let computed_count = self.computed_registry.lock()
            .map(|registry| registry.get_computed_count())
            .unwrap_or(0);

        let effect_count = self.effect_registry.lock()
            .map(|registry| registry.get_total_effect_count())
            .unwrap_or(0);

        let state = self.state.read()
            .map(|state| state.clone())
            .unwrap_or(RuntimeState::Shutdown);

        RuntimeStats {
            runtime_id: self.id,
            state,
            signal_count,
            computed_count,
            effect_count,
            uptime: self.created_at.elapsed(),
        }
    }

    pub fn pause(&self) -> Result<(), String> {
        if let Ok(mut state) = self.state.write() {
            match *state {
                RuntimeState::Running => {
                    *state = RuntimeState::Pausing;
                    // TODO: Actually pause processing
                    *state = RuntimeState::Paused;
                    Ok(())
                }
                _ => Err(format!("Cannot pause runtime in state: {:?}", *state))
            }
        } else {
            Err("Failed to acquire state lock for pause".to_string())
        }
    }

    pub fn resume(&self) -> Result<(), String> {
        if let Ok(mut state) = self.state.write() {
            match *state {
                RuntimeState::Paused => {
                    *state = RuntimeState::Running;
                    Ok(())
                }
                _ => Err(format!("Cannot resume runtime in state: {:?}", *state))
            }
        } else {
            Err("Failed to acquire state lock for resume".to_string())
        }
    }

    pub fn shutdown(&self) -> Result<(), String> {
        if let Ok(mut state) = self.state.write() {
            match *state {
                RuntimeState::Shutdown => return Ok(()),
                _ => *state = RuntimeState::ShuttingDown,
            }
        }

        // Dispose all effects first
        if let Ok(mut effect_registry) = self.effect_registry.lock() {
            effect_registry.dispose_all_effects();
        }

        // Clear all registries
        if let Ok(mut signal_registry) = self.signal_registry.lock() {
            signal_registry.signals.clear();
            signal_registry.signal_metadata.clear();
        }

        if let Ok(mut computed_registry) = self.computed_registry.lock() {
            computed_registry.computeds.clear();
            computed_registry.dependencies.clear();
        }

        // Signal shutdown to any listeners
        if let Ok(mut sender) = self.shutdown_sender.lock() {
            if let Some(sender) = sender.take() {
                let _ = sender.send(());
            }
        }

        // Update state to shutdown
        if let Ok(mut state) = self.state.write() {
            *state = RuntimeState::Shutdown;
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        matches!(
            self.state.read().map(|s| s.clone()).unwrap_or(RuntimeState::Shutdown),
            RuntimeState::Running
        )
    }

    pub fn get_batch_manager(&self) -> Arc<BatchManager> {
        self.batch_manager.clone()
    }

    pub fn get_frame_scheduler(&self) -> Arc<FrameScheduler> {
        self.frame_scheduler.clone()
    }

    pub fn id(&self) -> RuntimeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct CleanupStats {
    pub signals_removed: usize,
    pub computed_removed: usize,
    pub effects_disposed: usize,
}

#[derive(Debug, Clone)]
pub struct RuntimeStats {
    pub runtime_id: RuntimeId,
    pub state: RuntimeState,
    pub signal_count: usize,
    pub computed_count: usize,
    pub effect_count: usize,
    pub uptime: Duration,
}

impl Drop for ReactiveRuntime {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

static GLOBAL_RUNTIME: std::sync::OnceLock<(ReactiveRuntime, std::sync::Mutex<Option<oneshot::Receiver<()>>>)> = std::sync::OnceLock::new();

pub fn global_runtime() -> &'static ReactiveRuntime {
    &GLOBAL_RUNTIME.get_or_init(|| {
        let (runtime, shutdown_receiver) = ReactiveRuntime::new();
        let _ = runtime.initialize();
        (runtime, std::sync::Mutex::new(Some(shutdown_receiver)))
    }).0
}

pub fn take_global_runtime_shutdown_receiver() -> Option<oneshot::Receiver<()>> {
    GLOBAL_RUNTIME.get()
        .and_then(|(_, receiver)| receiver.lock().ok()?.take())
}

pub fn shutdown_global_runtime() -> Result<(), String> {
    if let Some((runtime, _)) = GLOBAL_RUNTIME.get() {
        runtime.shutdown()
    } else {
        Ok(()) // Already shutdown or never initialized
    }
}