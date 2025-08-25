use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::sync::{oneshot, watch};
use std::collections::HashSet;

use crate::{
    runtime::{global_runtime, shutdown_global_runtime},
    batch::global_batch_manager,
    frame_sync::global_frame_scheduler,
    threading::{global_thread_manager, ShutdownMessage}
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownReason {
    UserRequest,
    SystemShutdown,
    Error,
    Timeout,
    UnresponsiveUI,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownPhase {
    NotStarted,
    InitiatingShutdown,
    StoppingEffects,
    FlushingUpdates,
    CleaningResources,
    WaitingForThreads,
    ShutdownComplete,
    ShutdownFailed,
}

pub struct ShutdownCoordinator {
    phase: Arc<RwLock<ShutdownPhase>>,
    reason: Arc<RwLock<Option<ShutdownReason>>>,
    timeout: Duration,
    force_shutdown_sender: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    phase_watchers: Arc<Mutex<Vec<watch::Sender<ShutdownPhase>>>>,
    active_threads: Arc<Mutex<HashSet<usize>>>,
}

impl ShutdownCoordinator {
    pub fn new(timeout: Duration) -> (Self, oneshot::Receiver<()>) {
        let (force_sender, force_receiver) = oneshot::channel();
        
        let coordinator = Self {
            phase: Arc::new(RwLock::new(ShutdownPhase::NotStarted)),
            reason: Arc::new(RwLock::new(None)),
            timeout,
            force_shutdown_sender: Arc::new(Mutex::new(Some(force_sender))),
            phase_watchers: Arc::new(Mutex::new(Vec::new())),
            active_threads: Arc::new(Mutex::new(HashSet::new())),
        };
        
        (coordinator, force_receiver)
    }

    pub fn initiate_shutdown(&self, reason: ShutdownReason) -> Result<(), String> {
        // Set the shutdown reason
        if let Ok(mut current_reason) = self.reason.write() {
            if current_reason.is_some() {
                return Err("Shutdown already in progress".to_string());
            }
            *current_reason = Some(reason);
        } else {
            return Err("Failed to acquire reason lock".to_string());
        }

        // Begin shutdown sequence
        self.transition_to_phase(ShutdownPhase::InitiatingShutdown);

        // Send shutdown message to all threads
        let shutdown_msg = ShutdownMessage {
            reason: format!("{:?}", reason),
            force: false,
        };

        if let Err(_) = global_thread_manager().send_to_ui(shutdown_msg.clone()) {
            // UI thread might not be running, that's okay
        }
        if let Err(_) = global_thread_manager().send_to_main(shutdown_msg) {
            // Main thread might not be running, that's okay
        }

        // Start the shutdown process
        let coordinator = self.clone();
        tokio::spawn(async move {
            coordinator.execute_shutdown_sequence().await;
        });

        Ok(())
    }

    async fn execute_shutdown_sequence(&self) {
        // Phase 1: Stop all effects
        self.transition_to_phase(ShutdownPhase::StoppingEffects);
        self.stop_effects().await;

        // Phase 2: Flush any pending updates
        self.transition_to_phase(ShutdownPhase::FlushingUpdates);
        self.flush_pending_updates().await;

        // Phase 3: Clean up resources
        self.transition_to_phase(ShutdownPhase::CleaningResources);
        self.cleanup_resources().await;

        // Phase 4: Wait for threads to finish
        self.transition_to_phase(ShutdownPhase::WaitingForThreads);
        if self.wait_for_threads().await {
            self.transition_to_phase(ShutdownPhase::ShutdownComplete);
        } else {
            self.transition_to_phase(ShutdownPhase::ShutdownFailed);
            self.force_shutdown();
        }
    }

    async fn stop_effects(&self) {
        // Effects will be cleaned up by the runtime
        if let Err(e) = shutdown_global_runtime() {
            eprintln!("Error shutting down reactive runtime: {}", e);
        }
        
        // Give effects a moment to clean up
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    async fn flush_pending_updates(&self) {
        // Disable frame synchronization to allow immediate flushing
        global_frame_scheduler().enable_frame_sync(false);
        
        // Flush any pending batched updates
        global_batch_manager().flush_batch();
        
        // Wait a moment for any async operations to complete
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    async fn cleanup_resources(&self) {
        let runtime = global_runtime();
        let stats = runtime.cleanup_unused_resources();
        
        println!("Cleaned up {} signals during shutdown", stats.signals_removed);
        
        // Wait for cleanup to complete
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    async fn wait_for_threads(&self) -> bool {
        let timeout = tokio::time::sleep(self.timeout);
        tokio::pin!(timeout);

        loop {
            // Check if all threads have finished
            let active_count = if let Ok(threads) = self.active_threads.lock() {
                threads.len()
            } else {
                0
            };

            if active_count == 0 {
                return true;
            }

            tokio::select! {
                _ = &mut timeout => {
                    eprintln!("Timeout waiting for {} threads to finish", active_count);
                    return false;
                }
                _ = tokio::time::sleep(Duration::from_millis(50)) => {
                    // Continue checking
                }
            }
        }
    }

    fn force_shutdown(&self) {
        if let Ok(mut sender) = self.force_shutdown_sender.lock() {
            if let Some(sender) = sender.take() {
                let _ = sender.send(());
            }
        }
    }

    fn transition_to_phase(&self, new_phase: ShutdownPhase) {
        if let Ok(mut phase) = self.phase.write() {
            *phase = new_phase;
        }

        // Notify phase watchers
        if let Ok(mut watchers) = self.phase_watchers.lock() {
            watchers.retain(|watcher| watcher.send(new_phase).is_ok());
        }

        println!("Shutdown phase: {:?}", new_phase);
    }

    pub fn get_current_phase(&self) -> ShutdownPhase {
        self.phase.read()
            .map(|phase| *phase)
            .unwrap_or(ShutdownPhase::ShutdownFailed)
    }

    pub fn get_shutdown_reason(&self) -> Option<ShutdownReason> {
        self.reason.read()
            .map(|reason| *reason)
            .unwrap_or(None)
    }

    pub fn register_thread(&self, thread_id: usize) {
        if let Ok(mut threads) = self.active_threads.lock() {
            threads.insert(thread_id);
        }
    }

    pub fn unregister_thread(&self, thread_id: usize) {
        if let Ok(mut threads) = self.active_threads.lock() {
            threads.remove(&thread_id);
        }
    }

    pub fn watch_phase_changes(&self) -> watch::Receiver<ShutdownPhase> {
        let (sender, receiver) = watch::channel(self.get_current_phase());
        
        if let Ok(mut watchers) = self.phase_watchers.lock() {
            watchers.push(sender);
        }
        
        receiver
    }

    pub fn is_shutdown_complete(&self) -> bool {
        matches!(
            self.get_current_phase(),
            ShutdownPhase::ShutdownComplete | ShutdownPhase::ShutdownFailed
        )
    }

    pub fn is_shutting_down(&self) -> bool {
        !matches!(self.get_current_phase(), ShutdownPhase::NotStarted)
    }
}

impl Clone for ShutdownCoordinator {
    fn clone(&self) -> Self {
        Self {
            phase: self.phase.clone(),
            reason: self.reason.clone(),
            timeout: self.timeout,
            force_shutdown_sender: self.force_shutdown_sender.clone(),
            phase_watchers: self.phase_watchers.clone(),
            active_threads: self.active_threads.clone(),
        }
    }
}

static GLOBAL_SHUTDOWN_COORDINATOR: std::sync::OnceLock<(ShutdownCoordinator, std::sync::Mutex<Option<oneshot::Receiver<()>>>)> = std::sync::OnceLock::new();

pub fn global_shutdown_coordinator() -> &'static ShutdownCoordinator {
    &GLOBAL_SHUTDOWN_COORDINATOR.get_or_init(|| {
        let (coordinator, force_receiver) = ShutdownCoordinator::new(Duration::from_secs(5));
        (coordinator, std::sync::Mutex::new(Some(force_receiver)))
    }).0
}

pub fn take_global_force_shutdown_receiver() -> Option<oneshot::Receiver<()>> {
    GLOBAL_SHUTDOWN_COORDINATOR.get()
        .and_then(|(_, receiver)| receiver.lock().ok()?.take())
}

pub fn initiate_graceful_shutdown(reason: ShutdownReason) -> Result<(), String> {
    global_shutdown_coordinator().initiate_shutdown(reason)
}

pub fn register_shutdown_thread(thread_id: usize) {
    global_shutdown_coordinator().register_thread(thread_id);
}

pub fn unregister_shutdown_thread(thread_id: usize) {
    global_shutdown_coordinator().unregister_thread(thread_id);
}

pub fn is_shutting_down() -> bool {
    global_shutdown_coordinator().is_shutting_down()
}

pub fn watch_shutdown_progress() -> watch::Receiver<ShutdownPhase> {
    global_shutdown_coordinator().watch_phase_changes()
}

// Shutdown guard that automatically unregisters a thread on drop
pub struct ShutdownThreadGuard {
    thread_id: usize,
    coordinator: &'static ShutdownCoordinator,
}

impl ShutdownThreadGuard {
    pub fn new(thread_id: usize) -> Self {
        let coordinator = global_shutdown_coordinator();
        coordinator.register_thread(thread_id);
        
        Self {
            thread_id,
            coordinator,
        }
    }
}

impl Drop for ShutdownThreadGuard {
    fn drop(&mut self) {
        self.coordinator.unregister_thread(self.thread_id);
    }
}