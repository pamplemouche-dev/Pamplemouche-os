//! Inter-Process Communication — synchronous message-passing (Mach-port style).
//!
//! Each port holds a bounded FIFO queue of `Message` objects.  Tasks send
//! messages into a port by calling `send`; they receive by calling `recv`.
//! Ports are reference-counted and identified by a `PortRef` (an Arc).
//!
//! This is deliberately minimal: it demonstrates the micro-kernel IPC
//! primitive that higher-level syscalls (including Darwin mach_msg) will be
//! built on top of.

use alloc::{
    collections::VecDeque,
    string::String,
    sync::Arc,
    vec::Vec,
};
use spin::Mutex;

// ── Message ──────────────────────────────────────────────────────────────────

/// Maximum inline message payload (bytes).
pub const MAX_INLINE_PAYLOAD: usize = 256;

/// A kernel IPC message.
///
/// Large payloads should be transferred via shared-memory descriptors
/// (out-of-line data) rather than inline — that facility will be added
/// alongside the virtual-memory region API.
#[derive(Debug, Clone)]
pub struct Message {
    /// Sender task ID (0 = kernel).
    pub sender: u64,
    /// Message type discriminant — used by subsystems to demultiplex.
    pub msg_type: u32,
    /// Inline payload bytes.
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(sender: u64, msg_type: u32, payload: impl Into<Vec<u8>>) -> Self {
        let payload = payload.into();
        assert!(
            payload.len() <= MAX_INLINE_PAYLOAD,
            "inline payload exceeds {} bytes",
            MAX_INLINE_PAYLOAD
        );
        Message { sender, msg_type, payload }
    }
}

// ── Port ─────────────────────────────────────────────────────────────────────

/// Maximum number of messages a port can queue before senders are blocked.
const PORT_QUEUE_CAPACITY: usize = 64;

/// An IPC port — the fundamental communication endpoint.
pub struct Port {
    name: String,
    queue: VecDeque<Message>,
    capacity: usize,
}

impl Port {
    fn new(name: impl Into<String>) -> Self {
        Port {
            name: name.into(),
            queue: VecDeque::with_capacity(PORT_QUEUE_CAPACITY),
            capacity: PORT_QUEUE_CAPACITY,
        }
    }

    /// Enqueue a message.  Returns `Err(msg)` if the queue is full.
    fn enqueue(&mut self, msg: Message) -> Result<(), Message> {
        if self.queue.len() >= self.capacity {
            return Err(msg);
        }
        self.queue.push_back(msg);
        Ok(())
    }

    /// Dequeue the oldest message, if any.
    fn dequeue(&mut self) -> Option<Message> {
        self.queue.pop_front()
    }
}

// ── PortRef ───────────────────────────────────────────────────────────────────

/// A shared, thread-safe reference to a port.
pub type PortRef = Arc<Mutex<Port>>;

/// Allocate a new named port and return a reference to it.
pub fn create_port(name: impl Into<String>) -> PortRef {
    Arc::new(Mutex::new(Port::new(name)))
}

/// Send a message to a port.
///
/// Returns `Err(msg)` if the port queue is full (caller should retry or block).
pub fn send(port: &PortRef, msg: Message) -> Result<(), Message> {
    port.lock().enqueue(msg)
}

/// Receive the next message from a port.  Returns `None` if the port is empty.
pub fn recv(port: &PortRef) -> Option<Message> {
    port.lock().dequeue()
}

// ── Initialisation ────────────────────────────────────────────────────────────

/// Initialise the IPC subsystem.
pub fn init() {
    crate::serial_println!("[ipc] message-passing subsystem ready");
}
