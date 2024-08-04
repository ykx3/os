#![no_std]
#![no_main]

use lib::*;
use lib::sync::*;
use lib::collections::VecDeque;

extern crate lib;

static LEFT: Semaphore = Semaphore::new(0);
static RIGHT: Semaphore = Semaphore::new(1);
static SEM_PROD: Semaphore = Semaphore::new(2);
static SEM_CONS: Semaphore = Semaphore::new(3);

fn main() -> isize {
    LEFT.init(1);
    RIGHT.init(0);
    SEM_PROD.init(3);
    SEM_CONS.init(0);

    let mut cpids: [u16; 3] = [0; 3];
    for i in 0..3 {
        let pid = sys_fork();

        if pid == 0 {
            match i {
                0 => left(),
                1 => right(),
                2 => underline(),
                _ => (),
            }
        } else {
            cpids[i]=pid;
        }
    }

    for i in 0..3 {
        sys_wait_pid(cpids[i]);
    }

    SEM_CONS.remove();
    SEM_PROD.remove();

    0
}

fn left() {
    loop {
        SEM_PROD.wait(); 
        RIGHT.wait();
        print!("<");
        LEFT.signal();
        SEM_CONS.signal(); 
    }
}

fn right() {
    loop {
        SEM_PROD.wait(); 
        LEFT.wait();
        print!(">");
        RIGHT.signal();
        SEM_CONS.signal(); 
    }
}

fn underline() {
    loop {
        for _ in 0..3 {
            SEM_CONS.wait();
        }
        print!("_");
        for _ in 0..3 {
            SEM_PROD.signal();
        }
    }
}

entry!(main);
