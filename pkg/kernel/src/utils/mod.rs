#[macro_use]
mod macros;
#[macro_use]
mod regs;

pub mod clock;
pub mod func;
pub mod logger;
pub mod resource;

pub use macros::*;
pub use regs::*;

use crate::proc::{*};
use self::manager::get_process_manager;
use alloc::format;
pub const fn get_ascii_header() -> &'static str {
    concat!(
        r"
__  __      __  _____            ____  _____
\ \/ /___ _/ /_/ ___/___  ____  / __ \/ ___/
 \  / __ `/ __/\__ \/ _ \/ __ \/ / / /\__ \
 / / /_/ / /_ ___/ /  __/ / / / /_/ /___/ /
/_/\__,_/\__//____/\___/_/ /_/\____//____/

                                       v",
        env!("CARGO_PKG_VERSION")
    )
}

pub fn new_test_thread(id: &str) -> ProcessId {
    let mut proc_data = ProcessData::new();
    proc_data.set_env("id", id);

    spawn_kernel_thread(
        func::test,
        format!("#{}_test", id),
        Some(proc_data),
    )
}

pub fn new_stack_test_thread() {
    let pid = spawn_kernel_thread(
        func::stack_test,
        alloc::string::String::from("stack"),
        None,
    );

    // wait for progress exit
    wait(pid);
}

fn wait(pid: ProcessId) {
    loop {
        // FIXME: try to get the status of the process

        // HINT: it's better to use the exit code
        /* FIXME: is the process exited? */
        let maneger = get_process_manager();
        if let None = maneger.check_proc(&pid) {
            x86_64::instructions::hlt();
        } else {
            break;
        }
    }
}
