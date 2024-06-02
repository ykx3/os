#![no_std]
#![no_main]

use lib::*;

extern crate lib;

const THREAD_COUNT: usize = 8;
static mut COUNTER1: isize = 0;//for spin
static mut COUNTER2: isize = 0;//for sem

static SPIN_LOCK: sync::SpinLock = sync::SpinLock::new();
static SEM: sync::Semaphore = sync::Semaphore::new(0);

fn main() -> isize {
    let pid = sys_fork();

    if pid == 0 {
        test_semaphore();
    } else {
        test_spin();
        sys_wait_pid(pid);
    }

    0
}

fn test_spin() -> isize {
    let mut pids = [0u16; THREAD_COUNT];

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc1();
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER1 result: {}", unsafe { COUNTER1 });

    0
}

fn do_counter_inc1() {
    for _ in 0..100 {
        // FIXME: protect the critical section
        SPIN_LOCK.acquire();
        inc_counter1();
        SPIN_LOCK.release();
    }
}

fn test_semaphore() -> isize {
    SEM.init(1);
    let mut pids = [0u16; THREAD_COUNT];

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc2();
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER2 result: {}", unsafe { COUNTER2 });

    SEM.remove();

    0
}

fn do_counter_inc2() {
    for _ in 0..100 {
        // FIXME: protect the critical section
        SEM.wait();
        inc_counter2();
        SEM.signal();
    }
}

/// Increment the counter
///
/// this function simulate a critical section by delay
/// DO NOT MODIFY THIS FUNCTION
fn inc_counter1() {
    unsafe {
        delay();
        let mut val = COUNTER1;
        delay();
        val += 1;
        delay();
        COUNTER1 = val;
    }
}
fn inc_counter2() {
    unsafe {
        delay();
        let mut val = COUNTER2;
        delay();
        val += 1;
        delay();
        COUNTER2 = val;
    }
}

#[inline(never)]
#[no_mangle]
fn delay() {
    for _ in 0..0x100 {
        core::hint::spin_loop();
    }
}

entry!(main);
