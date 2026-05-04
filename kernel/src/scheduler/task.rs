//! Task descriptor — represents a schedulable unit of work.

use alloc::boxed::Box;
use core::sync::atomic::{AtomicU64, Ordering};

/// Monotonically-increasing task-ID generator.
#[allow(dead_code)]
static NEXT_TID: AtomicU64 = AtomicU64::new(1);

/// Unique task identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    #[allow(dead_code)]
    fn next() -> Self {
        TaskId(NEXT_TID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Lifecycle state of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Eligible to run.
    Ready,
    /// Currently executing on the CPU.
    Running,
    /// Blocked waiting for an IPC message or I/O.
    #[allow(dead_code)]
    Blocked,
    /// Terminated; resources can be reclaimed.
    #[allow(dead_code)]
    Zombie,
}

/// Saved integer register state used for context switching.
///
/// Layout must match the push order in the assembly context-switch stub.
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct CpuContext {
    pub rsp: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
}

/// Represents a single task (kernel thread or user process).
pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
    #[allow(dead_code)]
    pub context: CpuContext,
    /// Kernel stack allocation — kept alive for the task's lifetime.
    #[allow(dead_code)]
    pub stack: Box<[u8; STACK_SIZE]>,
    /// Human-readable name for debugging.
    pub name: &'static str,
    /// Remaining time-slice ticks before preemption.
    pub ticks_remaining: u32,
}

/// Kernel stack size per task: 16 KiB.
const STACK_SIZE: usize = 4096 * 4;
/// Default time-slice length in timer ticks (~10 ms at 100 Hz).
pub const DEFAULT_TIMESLICE: u32 = 10;

impl Task {
    /// Allocate a new kernel task that will begin executing `entry`.
    #[allow(dead_code)]
    pub fn new(name: &'static str, entry: fn() -> !) -> Self {
        let mut stack = Box::new([0u8; STACK_SIZE]);

        // Place the entry-point address at the top of the stack so that when
        // the context-switch stub executes `ret` it jumps to `entry`.
        let stack_top = stack.as_mut_ptr() as usize + STACK_SIZE;
        let rsp = (stack_top - core::mem::size_of::<u64>()) as u64;
        unsafe {
            *(rsp as *mut usize) = entry as usize;
        }

        Task {
            id: TaskId::next(),
            state: TaskState::Ready,
            context: CpuContext {
                rsp,
                ..CpuContext::default()
            },
            stack,
            name,
            ticks_remaining: DEFAULT_TIMESLICE,
        }
    }
}

impl core::fmt::Debug for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Task({:?}, {:?}, {:?})", self.id, self.name, self.state)
    }
}
