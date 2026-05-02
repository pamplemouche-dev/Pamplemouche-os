//! Preemptive round-robin scheduler.
//!
//! The scheduler is driven by the timer IRQ (IRQ0 at 100 Hz).  On each tick
//! it decrements the current task's time-slice counter.  When it reaches zero
//! the task is moved to the back of the ready queue and the next task is
//! selected.
//!
//! Context switching is performed via an inline-assembly stub that saves all
//! callee-saved registers onto the current task's kernel stack and restores
//! those of the next task.

pub mod task;

use alloc::collections::VecDeque;
use spin::Mutex;
use task::{Task, TaskState, DEFAULT_TIMESLICE};

/// Global scheduler state.
pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

/// Round-robin run queue.
pub struct Scheduler {
    /// Tasks ready to run, ordered front-to-back by priority/FIFO.
    ready_queue: VecDeque<Task>,
    /// Index into `ready_queue` of the currently-running task, if any.
    current: Option<usize>,
    /// Total number of context switches performed (for diagnostics).
    pub switch_count: u64,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            ready_queue: VecDeque::new(),
            current: None,
            switch_count: 0,
        }
    }

    /// Enqueue a new task.
    #[allow(dead_code)]
    pub fn spawn(&mut self, task: Task) {
        crate::serial_println!("[sched] spawning task {:?}", task.name);
        self.ready_queue.push_back(task);
    }

    /// Called on every timer tick.  Returns `true` if a context switch should
    /// be performed (the assembly stub in the IRQ handler acts on this).
    pub fn tick(&mut self) -> bool {
        // Decrement the running task's timeslice.
        if let Some(idx) = self.current {
            if let Some(task) = self.ready_queue.get_mut(idx) {
                if task.ticks_remaining > 1 {
                    task.ticks_remaining -= 1;
                    return false; // No preemption yet.
                }
                // Time-slice expired — reset and rotate.
                task.ticks_remaining = DEFAULT_TIMESLICE;
                task.state = TaskState::Ready;
            }
        }
        self.select_next()
    }

    /// Select the next `Ready` task and record it as the current task.
    /// Returns `true` if a different task was selected (context switch needed).
    fn select_next(&mut self) -> bool {
        if self.ready_queue.is_empty() {
            return false;
        }

        // Rotate: move the front task to the back and pick the new front.
        if let Some(front) = self.ready_queue.pop_front() {
            self.ready_queue.push_back(front);
        }

        // Find the first Ready task.
        for (i, task) in self.ready_queue.iter_mut().enumerate() {
            if task.state == TaskState::Ready {
                task.state = TaskState::Running;
                let switched = self.current != Some(i);
                self.current = Some(i);
                if switched {
                    self.switch_count += 1;
                }
                return switched;
            }
        }
        false
    }

    /// Block the task identified by `id`, moving it to `Blocked` state.
    #[allow(dead_code)]
    pub fn block_current(&mut self) {
        if let Some(idx) = self.current {
            if let Some(task) = self.ready_queue.get_mut(idx) {
                task.state = TaskState::Blocked;
            }
        }
    }

    /// Unblock a task by its task ID.
    #[allow(dead_code)]
    pub fn unblock(&mut self, id: task::TaskId) {
        for task in self.ready_queue.iter_mut() {
            if task.id == id && task.state == TaskState::Blocked {
                task.state = TaskState::Ready;
                return;
            }
        }
    }
}

// ── Public kernel API ────────────────────────────────────────────────────────

/// Initialise the scheduler (currently a no-op; structure is ready at compile
/// time via `const fn`).
pub fn init() {
    crate::serial_println!("[sched] scheduler ready");
}

/// Called from the timer IRQ handler — may trigger a context switch.
pub fn tick() {
    // Avoid a deadlock: if the lock is contended we simply skip this tick.
    if let Some(mut sched) = SCHEDULER.try_lock() {
        sched.tick();
    }
}

/// Spawn a new kernel task.
#[allow(dead_code)]
pub fn spawn(task: Task) {
    SCHEDULER.lock().spawn(task);
}
