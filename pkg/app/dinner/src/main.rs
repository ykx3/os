#![no_std]
#![no_main]

use lib::*;
use lib::sync::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

extern crate lib;

static CHOPSTICK: [Semaphore; 5] = semaphore_array![0, 1, 2, 3, 4];
static EATING: Semaphore = Semaphore::new(5);

fn main() -> isize {
    for i in 0..5 {
        CHOPSTICK[i].init(1);
    }
    EATING.init(4);

    let time = 1000000000;

    let mut cpids: [u16; 5] = [0; 5];
    for i in 0..5 {
        let pid = sys_fork();

        if pid == 0 {
            philosopher(i, time);
        } else {
            cpids[i]=pid;
        }
    }
    sys_stat();

    for i in 0..5 {
        sys_wait_pid(cpids[i]);
    }

    sys_stat();

    EATING.remove();
    for i in 0..5 {
        CHOPSTICK[i].remove();
    }

    0
}

fn philosopher(idx: usize, time: u64) {
    let start = sys_time();
    // println!("philosopher#{} created in {}.", idx, start);
    while sys_time() - start < time {
        think(idx);
        EATING.wait();
        eat(idx);
        EATING.signal();
    }
    sys_exit(0);
}

fn random_time(idx: usize, mode: u64) -> u64 {
    let time = lib::sys_time();
    let mut rng = ChaCha20Rng::seed_from_u64(time as u64 + idx as u64);
    // (rng.gen::<u64>()%mode + 1) * 1000
    1000
}

fn think(idx: usize) {
    println!("{}:philosopher#{} start thinking...", sys_time(), idx);
    let time = random_time(idx, 20);
    sleep(time);
    println!("{}:philosopher#{} end thinking...", sys_time(), idx);
}

fn eat(idx: usize) {
    println!("{}:philosopher#{} is trying to pick...", sys_time(), idx);
    let time = random_time(idx, 3);
    sleep(time);

    let left = (idx + 4) % 5;
    let right = (idx + 1) % 5;
    // let left = if left < right {left} else {right};
    // let right = if left < right {right} else {left};
    //pick left
    CHOPSTICK[left].wait();
    println!("{}:philosopher#{} get his left one ({})...", sys_time(), idx, left);

    let time = random_time(idx, 2);
    sleep(time);
    //pick right
    CHOPSTICK[right].wait();
    println!("{}:philosopher#{} get his right one ({})...", sys_time(), idx, right);

    //eat
    let time = random_time(idx, 2);
    sleep(time);
    println!("{}:philosopher#{} is eating...", sys_time(), idx);
    let time = random_time(idx, 20);
    sleep(time);

    //put down
    CHOPSTICK[left].signal();
    CHOPSTICK[right].signal();
    let time = random_time(idx, 2);
    sleep(time);
    println!("{}:philosopher#{} put down his chopsticks...", sys_time(), idx);
}

pub fn sleep(millisecs: u64) {
    let start = sys_time();
    // println!("start sleep in {}", start);
    let mut current = start;
    while current - start < millisecs {
        current = sys_time();
    }
    // println!("wake up in {}", current);
}


entry!(main);
