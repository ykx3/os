use crate::{filesystem::ls, memory::gdt, proc::*};
use alloc::format;
use x86_64::{structures::idt::{InterruptDescriptorTable, InterruptStackFrame}, PrivilegeLevel};

// NOTE: import `ysos_syscall` package as `syscall_def` in Cargo.toml
use syscall_def::Syscall;

mod service;
use self::manager::get_process_manager;

use super::consts;

// FIXME: write syscall service handler in `service.rs`
use service::*;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    // FIXME: register syscall handler to IDT
    //        - standalone syscall stack
    //        - ring 3
    idt[consts::Interrupts::Syscall as u8]
    .set_handler_fn(syscall_handler)
    .set_stack_index(gdt::SYSCALL_IST_INDEX)
    .set_privilege_level(PrivilegeLevel::Ring3);
}

pub extern "C" fn syscall(mut context: ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        super::syscall::dispatcher(&mut context);
    });
}

as_handler!(syscall);

#[derive(Clone, Debug)]
pub struct SyscallArgs {
    pub syscall: Syscall,
    pub arg0: usize,
    pub arg1: usize,
    pub arg2: usize,
}

pub fn dispatcher(context: &mut ProcessContext) {
    let args = super::syscall::SyscallArgs::new(
        Syscall::from(context.regs.rax),
        context.regs.rdi,
        context.regs.rsi,
        context.regs.rdx,
    );

    // NOTE: you may want to trace syscall arguments
    // trace!("{}", args);

    match args.syscall {
        // fd: arg0 as u8, buf: &[u8] (ptr: arg1 as *const u8, len: arg2)
        Syscall::Read => { /* FIXME: read from fd & return length */
            let ret = sys_read(&args);
            context.set_rax(ret);
        },
        // fd: arg0 as u8, buf: &[u8] (ptr: arg1 as *const u8, len: arg2)
        Syscall::Write => { /* FIXME: write to fd & return length */
            let ret = sys_write(&args);
            context.set_rax(ret);
        },

        // None -> pid: u16
        Syscall::GetPid => { /* FIXME: get current pid */ 
            let ret = get_process_manager().current().pid().0 as u16;
            context.set_rax(ret as usize);
        },

        // None -> pid: u16 or 0 or -1
        Syscall::Fork => { 
            sys_fork(context);
        },

        // path: &str (ptr: arg0 as *const u8, len: arg1) -> pid: u16
        Syscall::Spawn => { /* FIXME: spawn process from name */
            let ret = spawn_process(&args);
            context.set_rax(ret);
        },
        // ret: arg0 as isize
        Syscall::Exit => { /* FIXME: exit process with retcode */
            exit_process(&args, context);
        },
        // pid: arg0 as u16 -> status: isize
        Syscall::WaitPid => { /* FIXME: check if the process is running or get retcode */
            let pid = ProcessId(args.arg0 as u16);
            // let manager = get_process_manager();
            // let ret = if let Some(ret) = manager.check_proc(&pid) {
            //     ret
            // }else{
            //     -1
            // };
            // context.set_rax(ret as usize);
            wait_pid(pid, context);
        },

        // op: u8, key: u32, val: usize -> ret: any
        Syscall::Sem => sys_sem(&args, context),
        
        // None -> Time
        Syscall::Time => {
            context.set_rax(sys_time());
        }

        // None
        Syscall::Stat => { /* FIXME: list processes */ 
            list_process();
        },
        // None
        Syscall::ListApp => { /* FIXME: list available apps */
            list_app();
        },

        // path: &str (arg0 as *const u8, arg1 as len)
        Syscall::ListDir => {
            let ptr = args.arg0 as *const u8;
            let len = args.arg1 as usize;
            unsafe{
                let buf = core::slice::from_raw_parts(ptr, len);
                let path = core::str::from_utf8_unchecked(&buf);
                ls(path);
            }
        }

        // path: &str (arg0 as *const u8, arg1 as len)
        Syscall::Cat => {
            let ptr = args.arg0 as *const u8;
            let len = args.arg1 as usize;
            unsafe{
                let buf = core::slice::from_raw_parts(ptr, len);
                let path = core::str::from_utf8_unchecked(&buf);
                crate::filesystem::cat(path);
            }
        }

        // ----------------------------------------------------
        // NOTE: following syscall examples are implemented
        // ----------------------------------------------------

        // layout: arg0 as *const Layout -> ptr: *mut u8
        Syscall::Allocate => context.set_rax(sys_allocate(&args)),
        // ptr: arg0 as *mut u8
        Syscall::Deallocate => sys_deallocate(&args),
        // Unknown
        Syscall::Unknown => warn!("Unhandled syscall: {:x?}", context.regs.rax),
    }
}

impl SyscallArgs {
    pub fn new(syscall: Syscall, arg0: usize, arg1: usize, arg2: usize) -> Self {
        Self {
            syscall,
            arg0,
            arg1,
            arg2,
        }
    }
}

impl core::fmt::Display for SyscallArgs {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "SYSCALL: {:<10} (0x{:016x}, 0x{:016x}, 0x{:016x})",
            format!("{:?}", self.syscall),
            self.arg0,
            self.arg1,
            self.arg2
        )
    }
}
