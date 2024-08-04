#![no_std]
#![no_main]

use lib::*;
use lib::sync::*;
use lib::collections::VecDeque;

extern crate lib;

static mut QUEUE: VecDeque<u64> = VecDeque::new();

static SPIN_LOCK: SpinLock = SpinLock::new();
static SEM_PROD: Semaphore = Semaphore::new(0);
static SEM_CONS: Semaphore = Semaphore::new(1);

static LEN_Q: usize = 16;

fn main() -> isize {
    SEM_PROD.init(LEN_Q);
    SEM_CONS.init(0);

    let mut cpids: [u16; 16] = [0; 16];
    for i in 0..16 {
        let pid = sys_fork();

        if pid == 0 {
            if i < 8 {
                producer(i)
            }else{
                consumer(i)
            }
        } else {
            cpids[i]=pid;
        }
    }
    sys_stat();
    println!("len of queue: {}", LEN_Q);

    for i in 0..16 {
        sys_wait_pid(cpids[i]);
    }

    sys_stat();
    show_queue();

    SEM_CONS.remove();
    SEM_PROD.remove();

    0
}

fn show_queue(){
    unsafe{ println!("queue: {:?}", QUEUE);}
}

fn producer(idx: usize) {
    for _ in 0..10 {
        SEM_PROD.wait(); // 等待有空位
        SPIN_LOCK.acquire();
        unsafe {
            let item = idx as u64;
            QUEUE.push_back(item); // 生产1个元素
            println!("thread#{} is producing... Push {}.\nNow there are {} items.",idx , item, QUEUE.len());
        }
        SPIN_LOCK.release();
        SEM_CONS.signal(); // 通知有元素可消费
    }
    sys_exit(0);
}

fn consumer(idx: usize) {
    for _ in 0..10 {
        SEM_CONS.wait(); // 等待有元素可消费
        SPIN_LOCK.acquire();
        unsafe {
            let item = QUEUE.pop_front().unwrap(); // 消费了1个元素
            println!("thread#{} is cunsuming... Got {}.\nNow there are {} items.",idx , item, QUEUE.len());
        }
        SPIN_LOCK.release();
        SEM_PROD.signal(); // 通知有空位
    }
    sys_exit(0);
}

entry!(main);
