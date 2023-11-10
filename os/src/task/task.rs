//! Types related to task management

use super::{TaskContext, MAX_SYSCALL_NUM};
use crate::timer::get_time_ms;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// Count syscall times
    pub syscall_times_list: [u32; MAX_SYSCALL_NUM],
    /// Start time
    pub start_time: usize,
    /// If the task has started
    pub is_started: bool,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}

impl TaskControlBlock {
    /// increase syscall count
    pub fn increase_syscall_count(&mut self, syscall_id: usize) -> bool {
        if syscall_id >= MAX_SYSCALL_NUM {
            return false;
        }
        self.syscall_times_list[syscall_id] += 1;
        true
    }
    /// set task start time
    pub fn set_start_time(&mut self) {
        self.start_time = get_time_ms();
        self.is_started = true;
    }
}
