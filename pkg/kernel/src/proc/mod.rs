mod context;
mod data;
pub mod manager;
mod paging;
mod pid;
mod process;
mod processor;
mod sync;

use alloc::string::ToString;
use manager::*;
use process::*;
use storage::FileSystem;
use sync::*;
use crate::filesystem::get_rootfs;
use crate::memory::PAGE_SIZE;

use xmas_elf::ElfFile;
use alloc::{string::String, sync::Arc, vec::Vec};
pub use context::ProcessContext;
pub use paging::PageTableContext;
pub use data::ProcessData;
pub use pid::ProcessId;

use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;

// 0xffff_ff00_0000_0000 is the kernel's address space
pub const STACK_MAX: u64 = 0x0000_4000_0000_0000;

pub const STACK_MAX_PAGES: u64 = 0x100000;
pub const STACK_MAX_SIZE: u64 = STACK_MAX_PAGES * PAGE_SIZE;
pub const STACK_START_MASK: u64 = !(STACK_MAX_SIZE - 1);
// [bot..0x2000_0000_0000..top..0x3fff_ffff_ffff]
// init stack
pub const STACK_DEF_PAGE: u64 = 1;
pub const STACK_DEF_SIZE: u64 = STACK_DEF_PAGE * PAGE_SIZE;
pub const STACK_INIT_BOT: u64 = STACK_MAX - STACK_DEF_SIZE;
pub const STACK_INIT_TOP: u64 = STACK_MAX - 8;
// [bot..0xffffff0100000000..top..0xffffff01ffffffff]
// kernel stack
pub const KSTACK_MAX: u64 = 0xffff_ff02_0000_0000;
pub const KSTACK_DEF_PAGE: u64 = 512 /* FIXME: decide on the boot config */;
pub const KSTACK_DEF_SIZE: u64 = KSTACK_DEF_PAGE * PAGE_SIZE;
pub const KSTACK_INIT_BOT: u64 = KSTACK_MAX - KSTACK_DEF_SIZE;
pub const KSTACK_INIT_TOP: u64 = KSTACK_MAX - 8;

pub const KERNEL_PID: ProcessId = ProcessId(1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Running,
    Ready,
    Blocked,
    Dead,
}

/// init process manager
pub fn init(boot_info: &'static boot::BootInfo) {
    let mut kproc_data = ProcessData::new();

    // FIXME: set the kernel stack
    kproc_data.set_stack(VirtAddr::new(KSTACK_INIT_BOT), KSTACK_DEF_PAGE);
    trace!("Init process data: {:#?}", kproc_data);

    // kernel process
    let kproc = { 
        /* FIXME: create kernel process */ 
        Process::new(String::from("kernel"), None, PageTableContext::new(), Some(kproc_data))
    };
    let app_list = boot_info.loaded_apps.as_ref();
    manager::init(kproc, app_list);

    info!("Process Manager Initialized.");
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: switch to the next process
        // info!("in switch");
        let manager = get_process_manager();
        manager.save_current(context);
        manager.switch_next(context);
    });
}

pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // info!("spawn");
        let entry = VirtAddr::new(entry as usize as u64);
        get_process_manager().spawn_kernel_thread(entry, name, data)
    })
}

pub fn spawn(path: &str) -> Option<ProcessId> {
    let parts: Vec<&str> = path.split('/').collect();
    let name = parts.last().unwrap();

    let fs = get_rootfs();
    let f = fs.open_file(path);

    let mut buf = Vec::new(); 
    let elf = match f {
        Ok(mut file) => {
            let _ = file.read_all(&mut buf);
            xmas_elf::ElfFile::new(&buf)
        },
        Err(_) => {
            Err("Error opening file")
        }, 
    }.unwrap();

    elf_spawn(name.to_string(), &elf)
}

pub fn elf_spawn(name: String, elf: &ElfFile) -> Option<ProcessId> {
    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let process_name = name.to_lowercase();
        let parent = Arc::downgrade(&manager.current());
        let pid = manager.spawn(elf, name, Some(parent), None);

        debug!("Spawned process: {}#{}", process_name, pid);
        pid
    });

    Some(pid)
}

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // info!("list");
        get_process_manager().print_process_list();
    })
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: get current process's environment variable
        // info!("env");
        get_process_manager().current().read().env(key)
    })
}

pub fn process_exit(ret: isize) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // info!("exit");
        get_process_manager().kill_current(ret);
    });

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // info!("page");
        get_process_manager().handle_page_fault(addr, err_code)
    })
}

pub fn list_app() {
    let fs = get_rootfs();
    let iter = fs.read_dir("/app");
    let apps = iter.unwrap()
        .filter(|meta| meta.is_file())
        .map(|meta| meta.name.clone())  
        .collect::<Vec<String>>()       
        .join(", "); 
    println!("[+] App list: {}", apps);
}

pub fn read(fd: u8, mut buf: &mut [u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().current().write().read(fd, &mut buf))
}

pub fn write(fd: u8, buf: &[u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().current().write().write(fd, &buf))
}

pub fn exit(ret: isize, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // FIXME: implement this for ProcessManager
        manager.kill_self(ret);
        manager.switch_next(context);
    })
}

#[inline]
pub fn still_alive(pid: ProcessId) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // check if the process is still alive
        // print_process_list();
        get_process_manager().get_proc(&pid).unwrap().read().status() != ProgramStatus::Dead
    })
}

pub fn fork(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // FIXME: save_current as parent
        manager.save_current(context);
        // FIXME: fork to get child
        manager.fork();
        // FIXME: push to child & parent to ready queue
        // FIXME: switch to next process
        manager.switch_next(context);
    })
}

pub fn wait_pid(pid: ProcessId, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        if let Some(ret) = manager.check_proc(&pid) {
            context.set_rax(ret as usize);
        } else {
            manager.wait_pid(pid);
            manager.save_current(context);
            manager.current().write().block();
            manager.switch_next(context);
        }
    })
}

pub fn new_sem(key: u32, value: usize) -> usize{
    let manager = get_process_manager();
    let now = manager.current();
    if now.write().sem_new(key, value) {
        0
    }else {
        1
    }
}

pub fn remove_sem(key: u32) -> usize{
    let manager = get_process_manager();
    let now = manager.current();
    if now.write().sem_remove(key) {
        0
    }else {
        1
    }
}

pub fn sem_signal(key: u32, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let ret = manager.current().write().sem_signal(key);
        match ret {
            SemaphoreResult::Ok => context.set_rax(0),
            SemaphoreResult::NotExist => context.set_rax(1),
            SemaphoreResult::WakeUp(pid) => {
                // FIXME: 与 wait_pid 系统调用类似，你需要在 sem_signal 中对进程进行唤醒。
                // 但是此处无需为进程设置返回值，因此在调用 wake_up 时，传入 None 即可。
                manager.wake_up(pid, None);
            }
            _ => unreachable!(),
        }
    })
}

pub fn sem_wait(key: u32, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let pid = manager.current().pid();
        let ret = manager.current().write().sem_wait(key, pid);
        match ret {
            SemaphoreResult::Ok => context.set_rax(0),
            SemaphoreResult::NotExist => context.set_rax(1),
            SemaphoreResult::Block(pid) => {
                // FIXME: save, block it, then switch to next
                //        use `save_current` and `switch_next`
                // info!("111");
                manager.save_current(context);
                let proc = manager.get_proc(&pid).unwrap();
                proc.write().block();
                manager.switch_next(context);
            }
            _ => unreachable!(),
        }
    })
}