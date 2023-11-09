//! Process management syscalls
use core::mem::size_of;

use crate::{
    config::PAGE_SIZE,
    mm::{MapPermission, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        get_current_task_start_time, get_current_task_syscall_times, mmap, munmap,
        suspend_current_and_run_next, TaskStatus,
    },
};
use alloc::vec::Vec;

use crate::{
    config::MAX_SYSCALL_NUM,
    mm::translated_byte_buffer,
    timer::{get_time_ms, get_time_us},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// convert any data type to u8 slice
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

fn write_u8_slice(src: &[u8], dst: Vec<&mut [u8]>) {
    let mut s = 0;
    for part in dst {
        for elem in part {
            *elem = src[s];
            s += 1;
        }
    }
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let res = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let res_slice = unsafe { any_as_u8_slice(&res) };
    let ts = translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());
    // info!("sys_get_time: start to write result!");
    write_u8_slice(res_slice, ts);
    // info!("sys_get_time: write result succeed!");
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let syscall_times = get_current_task_syscall_times();
    let start_time = get_current_task_start_time();
    let res = TaskInfo {
        status: TaskStatus::Running,
        syscall_times,
        time: get_time_ms() - start_time,
    };
    let res_slice = unsafe { any_as_u8_slice(&res) };
    let ti = translated_byte_buffer(current_user_token(), ti as *const u8, size_of::<TaskInfo>());
    write_u8_slice(res_slice, ti);
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % PAGE_SIZE != 0 {
        // start pos not aligned
        return -1;
    } else if port & !0x7 != 0 {
        // only the least 3 bits of port can be non-zero
        return -1;
    } else if port & 0x7 == 0 {
        // no mode set
        return -1;
    }
    let port: u8 = port.try_into().unwrap();
    let page_num = len.div_ceil(PAGE_SIZE);
    let start_va = VirtAddr(start);
    let end_va = VirtAddr(start + page_num * PAGE_SIZE);
    let permission = MapPermission::from_bits_truncate(port << 1) | MapPermission::U;
    mmap(start_va, end_va, permission)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % PAGE_SIZE != 0 {
        // start pos not aligned
        return -1;
    }
    let page_num = len.div_ceil(PAGE_SIZE);
    let start_va = VirtAddr(start);
    let end_va = VirtAddr(start + page_num * PAGE_SIZE);
    munmap(start_va, end_va)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
