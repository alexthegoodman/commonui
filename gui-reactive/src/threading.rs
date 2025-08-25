use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use tokio::sync::{mpsc, broadcast};
use std::collections::HashMap;

pub type ThreadId = usize;
pub type MessageId = u64;

#[derive(Debug, Clone)]
pub enum ThreadType {
    Main,
    UI,
    Worker(String),
}

#[derive(Debug, Clone)]
pub struct ThreadInfo {
    pub id: ThreadId,
    pub thread_type: ThreadType,
    pub name: String,
}

pub trait Message: Send + Sync + 'static {
    fn message_type(&self) -> &'static str;
    fn priority(&self) -> MessagePriority;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

struct PrioritizedMessage {
    id: MessageId,
    priority: MessagePriority,
    payload: Box<dyn Message>,
    sender_id: ThreadId,
    timestamp: std::time::Instant,
}

impl PartialEq for PrioritizedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.id == other.id
    }
}

impl Eq for PrioritizedMessage {}

impl PartialOrd for PrioritizedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then older messages first
        other.priority.cmp(&self.priority)
            .then_with(|| self.timestamp.cmp(&other.timestamp))
    }
}

pub struct MessageChannel<T: Message> {
    sender: broadcast::Sender<T>,
    _receiver: broadcast::Receiver<T>,
}

impl<T: Message + Clone> MessageChannel<T> {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = broadcast::channel(capacity);
        Self {
            sender,
            _receiver: receiver,
        }
    }

    pub fn send(&self, message: T) -> Result<usize, broadcast::error::SendError<T>> {
        self.sender.send(message)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }

    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl<T: Message + Clone> Clone for MessageChannel<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            _receiver: self.sender.subscribe(),
        }
    }
}

pub struct ThreadManager {
    threads: Arc<RwLock<HashMap<ThreadId, ThreadInfo>>>,
    message_queue: Arc<Mutex<std::collections::BinaryHeap<PrioritizedMessage>>>,
    next_thread_id: Arc<Mutex<ThreadId>>,
    next_message_id: Arc<Mutex<MessageId>>,
    shutdown_signal: Arc<RwLock<bool>>,
    main_to_ui_channel: mpsc::UnboundedSender<Box<dyn Message>>,
    ui_to_main_channel: mpsc::UnboundedSender<Box<dyn Message>>,
}

impl ThreadManager {
    pub fn new() -> (Self, ThreadChannels) {
        let (main_to_ui_tx, main_to_ui_rx) = mpsc::unbounded_channel();
        let (ui_to_main_tx, ui_to_main_rx) = mpsc::unbounded_channel();

        let manager = Self {
            threads: Arc::new(RwLock::new(HashMap::new())),
            message_queue: Arc::new(Mutex::new(std::collections::BinaryHeap::new())),
            next_thread_id: Arc::new(Mutex::new(0)),
            next_message_id: Arc::new(Mutex::new(0)),
            shutdown_signal: Arc::new(RwLock::new(false)),
            main_to_ui_channel: main_to_ui_tx,
            ui_to_main_channel: ui_to_main_tx,
        };

        let channels = ThreadChannels {
            main_to_ui_rx,
            ui_to_main_rx,
            main_to_ui_tx: manager.main_to_ui_channel.clone(),
            ui_to_main_tx: manager.ui_to_main_channel.clone(),
        };

        (manager, channels)
    }

    pub fn register_thread(&self, thread_type: ThreadType, name: String) -> ThreadId {
        let thread_id = {
            let mut next_id = self.next_thread_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let thread_info = ThreadInfo {
            id: thread_id,
            thread_type,
            name,
        };

        if let Ok(mut threads) = self.threads.write() {
            threads.insert(thread_id, thread_info);
        }

        thread_id
    }

    pub fn unregister_thread(&self, thread_id: ThreadId) {
        if let Ok(mut threads) = self.threads.write() {
            threads.remove(&thread_id);
        }
    }

    pub fn send_to_ui<T: Message>(&self, message: T) -> Result<(), mpsc::error::SendError<Box<dyn Message>>> {
        self.main_to_ui_channel.send(Box::new(message))
    }

    pub fn send_to_main<T: Message>(&self, message: T) -> Result<(), mpsc::error::SendError<Box<dyn Message>>> {
        self.ui_to_main_channel.send(Box::new(message))
    }

    pub fn queue_prioritized_message(&self, message: Box<dyn Message>, sender_id: ThreadId) {
        if let Ok(mut queue) = self.message_queue.lock() {
            let message_id = {
                let mut next_id = self.next_message_id.lock().unwrap();
                let id = *next_id;
                *next_id += 1;
                id
            };

            let prioritized = PrioritizedMessage {
                id: message_id,
                priority: message.priority(),
                payload: message,
                sender_id,
                timestamp: std::time::Instant::now(),
            };

            queue.push(prioritized);
        }
    }

    pub fn process_next_message(&self) -> Option<(Box<dyn Message>, ThreadId)> {
        if let Ok(mut queue) = self.message_queue.lock() {
            queue.pop().map(|msg| (msg.payload, msg.sender_id))
        } else {
            None
        }
    }

    pub fn process_all_messages<F>(&self, mut handler: F) -> usize 
    where
        F: FnMut(Box<dyn Message>, ThreadId),
    {
        let mut count = 0;
        while let Some((message, sender_id)) = self.process_next_message() {
            handler(message, sender_id);
            count += 1;
        }
        count
    }

    pub fn get_thread_info(&self, thread_id: ThreadId) -> Option<ThreadInfo> {
        self.threads.read().ok()?.get(&thread_id).cloned()
    }

    pub fn list_threads(&self) -> Vec<ThreadInfo> {
        self.threads.read()
            .map(|threads| threads.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn signal_shutdown(&self) {
        if let Ok(mut shutdown) = self.shutdown_signal.write() {
            *shutdown = true;
        }
    }

    pub fn should_shutdown(&self) -> bool {
        self.shutdown_signal.read()
            .map(|shutdown| *shutdown)
            .unwrap_or(false)
    }

    pub fn message_queue_size(&self) -> usize {
        self.message_queue.lock()
            .map(|queue| queue.len())
            .unwrap_or(0)
    }
}

impl Default for ThreadManager {
    fn default() -> Self {
        Self::new().0
    }
}

pub struct ThreadChannels {
    main_to_ui_rx: mpsc::UnboundedReceiver<Box<dyn Message>>,
    ui_to_main_rx: mpsc::UnboundedReceiver<Box<dyn Message>>,
    main_to_ui_tx: mpsc::UnboundedSender<Box<dyn Message>>,
    ui_to_main_tx: mpsc::UnboundedSender<Box<dyn Message>>,
}

impl ThreadChannels {
    pub fn take_main_to_ui_receiver(self) -> (mpsc::UnboundedReceiver<Box<dyn Message>>, ThreadChannelsSenders) {
        (
            self.main_to_ui_rx,
            ThreadChannelsSenders {
                ui_to_main_rx: self.ui_to_main_rx,
                main_to_ui_tx: self.main_to_ui_tx,
                ui_to_main_tx: self.ui_to_main_tx,
            }
        )
    }
}

pub struct ThreadChannelsSenders {
    ui_to_main_rx: mpsc::UnboundedReceiver<Box<dyn Message>>,
    main_to_ui_tx: mpsc::UnboundedSender<Box<dyn Message>>,
    ui_to_main_tx: mpsc::UnboundedSender<Box<dyn Message>>,
}

impl ThreadChannelsSenders {
    pub fn take_ui_to_main_receiver(self) -> (mpsc::UnboundedReceiver<Box<dyn Message>>, ThreadSenders) {
        (
            self.ui_to_main_rx,
            ThreadSenders {
                main_to_ui_tx: self.main_to_ui_tx,
                ui_to_main_tx: self.ui_to_main_tx,
            }
        )
    }
}

pub struct ThreadSenders {
    main_to_ui_tx: mpsc::UnboundedSender<Box<dyn Message>>,
    ui_to_main_tx: mpsc::UnboundedSender<Box<dyn Message>>,
}

impl ThreadSenders {
    pub fn send_to_ui<T: Message>(&self, message: T) -> Result<(), mpsc::error::SendError<Box<dyn Message>>> {
        self.main_to_ui_tx.send(Box::new(message))
    }

    pub fn send_to_main<T: Message>(&self, message: T) -> Result<(), mpsc::error::SendError<Box<dyn Message>>> {
        self.ui_to_main_tx.send(Box::new(message))
    }
}

static GLOBAL_THREAD_MANAGER: std::sync::OnceLock<(ThreadManager, std::sync::Mutex<Option<ThreadChannels>>)> = std::sync::OnceLock::new();

pub fn global_thread_manager() -> &'static ThreadManager {
    &GLOBAL_THREAD_MANAGER.get_or_init(|| {
        let (manager, channels) = ThreadManager::new();
        (manager, std::sync::Mutex::new(Some(channels)))
    }).0
}

pub fn take_global_thread_channels() -> Option<ThreadChannels> {
    GLOBAL_THREAD_MANAGER.get()
        .and_then(|(_, channels)| channels.lock().ok()?.take())
}

// Common message types for UI communication
#[derive(Debug, Clone)]
pub struct RenderMessage {
    pub width: u32,
    pub height: u32,
    pub force_redraw: bool,
}

impl Message for RenderMessage {
    fn message_type(&self) -> &'static str {
        "render"
    }

    fn priority(&self) -> MessagePriority {
        if self.force_redraw {
            MessagePriority::High
        } else {
            MessagePriority::Normal
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputMessage {
    pub event_type: String,
    pub data: Vec<u8>,
}

impl Message for InputMessage {
    fn message_type(&self) -> &'static str {
        "input"
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::High
    }
}

#[derive(Debug, Clone)]
pub struct StateMessage {
    pub signal_id: usize,
    pub new_value: Vec<u8>, // Serialized value
}

impl Message for StateMessage {
    fn message_type(&self) -> &'static str {
        "state"
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::Normal
    }
}

#[derive(Debug, Clone)]
pub struct ShutdownMessage {
    pub reason: String,
    pub force: bool,
}

impl Message for ShutdownMessage {
    fn message_type(&self) -> &'static str {
        "shutdown"
    }

    fn priority(&self) -> MessagePriority {
        if self.force {
            MessagePriority::Critical
        } else {
            MessagePriority::High
        }
    }
}