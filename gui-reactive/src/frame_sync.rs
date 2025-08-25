use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use crate::batch::global_batch_manager;

type FrameCallback = Box<dyn FnOnce() + Send>;

pub struct FrameScheduler {
    frame_callbacks: Arc<Mutex<VecDeque<FrameCallback>>>,
    next_frame_callbacks: Arc<Mutex<VecDeque<FrameCallback>>>,
    frame_sync_enabled: Arc<RwLock<bool>>,
    target_fps: Arc<RwLock<u32>>,
    frame_time: Arc<RwLock<Duration>>,
    last_frame: Arc<Mutex<Instant>>,
    frame_counter: Arc<Mutex<u64>>,
}

impl FrameScheduler {
    pub fn new(target_fps: u32) -> Self {
        let frame_time = Duration::from_millis(1000 / target_fps as u64);
        
        Self {
            frame_callbacks: Arc::new(Mutex::new(VecDeque::new())),
            next_frame_callbacks: Arc::new(Mutex::new(VecDeque::new())),
            frame_sync_enabled: Arc::new(RwLock::new(true)),
            target_fps: Arc::new(RwLock::new(target_fps)),
            frame_time: Arc::new(RwLock::new(frame_time)),
            last_frame: Arc::new(Mutex::new(Instant::now())),
            frame_counter: Arc::new(Mutex::new(0)),
        }
    }

    pub fn schedule_for_next_frame<F>(&self, callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Ok(mut callbacks) = self.next_frame_callbacks.lock() {
            callbacks.push_back(Box::new(callback));
        }
    }

    pub fn schedule_for_current_frame<F>(&self, callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Ok(mut callbacks) = self.frame_callbacks.lock() {
            callbacks.push_back(Box::new(callback));
        }
    }

    pub fn begin_frame(&self) -> FrameContext {
        if let Ok(mut last_frame) = self.last_frame.lock() {
            *last_frame = Instant::now();
        }

        if let Ok(mut counter) = self.frame_counter.lock() {
            *counter += 1;
        }

        // Move next frame callbacks to current frame
        if let (Ok(mut current), Ok(mut next)) = (
            self.frame_callbacks.lock(),
            self.next_frame_callbacks.lock(),
        ) {
            while let Some(callback) = next.pop_front() {
                current.push_back(callback);
            }
        }

        // Start a new batch for this frame
        if self.is_frame_sync_enabled() {
            global_batch_manager().start_batch();
        }

        FrameContext {
            scheduler: self,
            started_batch: self.is_frame_sync_enabled(),
        }
    }

    pub fn end_frame(&self, context: FrameContext) {
        // Execute all frame callbacks
        if let Ok(mut callbacks) = self.frame_callbacks.lock() {
            while let Some(callback) = callbacks.pop_front() {
                callback();
            }
        }

        // Flush any batched signal updates
        if context.started_batch {
            global_batch_manager().flush_batch();
        }
    }

    pub fn set_target_fps(&self, fps: u32) {
        if let (Ok(mut target), Ok(mut frame_time)) = (
            self.target_fps.write(),
            self.frame_time.write(),
        ) {
            *target = fps;
            *frame_time = Duration::from_millis(1000 / fps as u64);
        }
    }

    pub fn get_target_fps(&self) -> u32 {
        self.target_fps.read().map(|fps| *fps).unwrap_or(60)
    }

    pub fn get_frame_time(&self) -> Duration {
        self.frame_time.read().map(|time| *time).unwrap_or(Duration::from_millis(16))
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame_counter.lock().map(|counter| *counter).unwrap_or(0)
    }

    pub fn enable_frame_sync(&self, enabled: bool) {
        if let Ok(mut sync_enabled) = self.frame_sync_enabled.write() {
            *sync_enabled = enabled;
        }
    }

    pub fn is_frame_sync_enabled(&self) -> bool {
        self.frame_sync_enabled.read().map(|enabled| *enabled).unwrap_or(false)
    }

    pub fn wait_for_frame_time(&self) {
        if let Ok(last_frame) = self.last_frame.lock() {
            let elapsed = last_frame.elapsed();
            let target_time = self.get_frame_time();
            
            if elapsed < target_time {
                let sleep_time = target_time - elapsed;
                std::thread::sleep(sleep_time);
            }
        }
    }

    pub async fn wait_for_frame_time_async(&self) {
        if let Ok(last_frame) = self.last_frame.lock() {
            let elapsed = last_frame.elapsed();
            let target_time = self.get_frame_time();
            
            if elapsed < target_time {
                let sleep_time = target_time - elapsed;
                tokio::time::sleep(sleep_time).await;
            }
        }
    }
}

impl Default for FrameScheduler {
    fn default() -> Self {
        Self::new(60) // 60 FPS default
    }
}

pub struct FrameContext<'a> {
    scheduler: &'a FrameScheduler,
    started_batch: bool,
}

impl<'a> Drop for FrameContext<'a> {
    fn drop(&mut self) {
        // Ensure end_frame is called even if not done explicitly
        if self.started_batch {
            global_batch_manager().flush_batch();
        }
    }
}

static GLOBAL_FRAME_SCHEDULER: std::sync::OnceLock<FrameScheduler> = std::sync::OnceLock::new();

pub fn global_frame_scheduler() -> &'static FrameScheduler {
    GLOBAL_FRAME_SCHEDULER.get_or_init(|| FrameScheduler::new(60))
}

pub fn schedule_for_next_frame<F>(callback: F)
where
    F: FnOnce() + Send + 'static,
{
    global_frame_scheduler().schedule_for_next_frame(callback);
}

pub fn schedule_for_current_frame<F>(callback: F)
where
    F: FnOnce() + Send + 'static,
{
    global_frame_scheduler().schedule_for_current_frame(callback);
}

pub struct FrameSynchronizedSignal<T> {
    inner: crate::signal::Signal<T>,
}

impl<T> FrameSynchronizedSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(initial_value: T) -> Self {
        Self {
            inner: crate::signal::Signal::new(initial_value),
        }
    }

    pub fn from_signal(signal: crate::signal::Signal<T>) -> Self {
        Self { inner: signal }
    }

    pub fn get(&self) -> T {
        self.inner.get()
    }

    pub fn set(&self, new_value: T) {
        if global_frame_scheduler().is_frame_sync_enabled() {
            let signal = self.inner.clone();
            let value = new_value.clone();
            schedule_for_next_frame(move || signal.set(value));
        } else {
            self.inner.set(new_value);
        }
    }

    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T) + Send + 'static,
    {
        if global_frame_scheduler().is_frame_sync_enabled() {
            let signal = self.inner.clone();
            schedule_for_next_frame(move || signal.update(updater));
        } else {
            self.inner.update(updater);
        }
    }

    pub fn set_immediate(&self, new_value: T) {
        self.inner.set(new_value);
    }

    pub fn update_immediate<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        self.inner.update(updater);
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.inner.with(f)
    }

    pub fn id(&self) -> crate::signal::SignalId {
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

    pub fn into_signal(self) -> crate::signal::Signal<T> {
        self.inner
    }
}

impl<T> Clone for FrameSynchronizedSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> From<crate::signal::Signal<T>> for FrameSynchronizedSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn from(signal: crate::signal::Signal<T>) -> Self {
        Self::from_signal(signal)
    }
}

impl<T> From<FrameSynchronizedSignal<T>> for crate::signal::Signal<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn from(frame_sync: FrameSynchronizedSignal<T>) -> Self {
        frame_sync.into_signal()
    }
}