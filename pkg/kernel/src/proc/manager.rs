use core::fmt::Debug;

use super::*;
use crate::memory::{
    self, allocator::{ALLOCATOR, HEAP_SIZE}, get_frame_alloc_for_sure, user, PAGE_SIZE
};
use alloc::{collections::*, format, sync::Weak};
use boot::AppListRef;
use elf::load_elf;
use spin::{Mutex, RwLock};
use alloc::sync::Arc;
use x86::task::tr;

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, app_list: boot::AppListRef) {

    // FIXME: set init process as Running
    init.write().resume();
    // FIXME: set processor's current pid to init's pid
    processor::set_pid(init.pid());
    PROCESS_MANAGER.call_once(|| ProcessManager::new(init, app_list));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    app_list: boot::AppListRef,
}

impl ProcessManager {
    pub fn new(init: Arc<Process>, app_list: boot::AppListRef) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
            app_list: app_list,
        }
    }

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    pub fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn app_list(&self) -> AppListRef{
        self.app_list
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn check_proc(&self, pid:&ProcessId) -> Option<isize>{
        self.get_proc(pid).unwrap().read().exit_code()
    }

    pub fn save_current(&self, context: &ProcessContext) {
        // FIXME: update current process's tick count

        // FIXME: update current process's context

        // FIXME: push current process to ready queue if still alive
        let now = self.current();
        let mut inner = now.write();
        if inner.status() != ProgramStatus::Dead {
            inner.tick();
            inner.save(& context);
            inner.pause();
            self.push_ready(now.pid());
        }
        // info!("saved {}",now.pid().0)   
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {

        // FIXME: fetch the next process from ready queue

        // FIXME: check if the next process is ready,
        //        continue to fetch if not ready

        // FIXME: restore next process's context

        // FIXME: update processor's current pid

        // FIXME: return next process's pid
        let mut ready = self.ready_queue.lock();
        while !ready.is_empty(){
            let pid = ready.pop_front().unwrap();
            let new = self.get_proc(&pid).unwrap();
            let mut new_inner = new.write();
            if new_inner.is_ready() {
                new_inner.resume();
                new_inner.restore(context);
                set_pid(new.pid());
                // info!("switch to {}",new.pid().0);
                return new.pid();
            }
        }
        panic!("no thread ready!")
    }

    pub fn spawn_kernel_thread(
        &self,
        entry: VirtAddr,
        name: String,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        panic!("bad function!");
        // // info!("spawn");
        // let kproc = self.get_proc(&KERNEL_PID).unwrap();
        // let page_table = kproc.read().clone_page_table();
        // let proc = Process::new(name, Some(Arc::downgrade(&kproc)), page_table, proc_data);

        // // alloc stack for the new process base on pid
        // let stack_top = proc.alloc_init_stack();

        // // FIXME: set the stack frame
        // proc.write().init_stack(entry, stack_top);
        // // FIXME: add to process map
        // let pid = proc.pid();
        // self.processes.write().insert(pid, proc);
        // // FIXME: push to ready queue
        // self.ready_queue.lock().push_back(pid);
        // // FIXME: return new process pid
        // pid
    }

    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc = Process::new(name, parent, page_table, proc_data);
        let pid = proc.pid();
        debug!("spawning");
        {      
            let inner = proc.write(); 
            // FIXME: load elf to process pagetable
            let frame_allocator = &mut *get_frame_alloc_for_sure();
            let mut page_table = inner.page_table.as_ref().unwrap().mapper();
            
            let _ = load_elf(elf, 0xFFFF800000000000, &mut page_table, frame_allocator, true);//notice
        }
        // FIXME: alloc new stack for process
        // alloc stack for the new process base on pid
        let stack_top = proc.alloc_init_stack(true);
        let entry = elf.header.pt2.entry_point();
        // FIXME: set the stack frame
        proc.write().init_stack(VirtAddr::new(entry), stack_top);
    
        trace!("New {:#?}", &proc);
    
        // FIXME: something like kernel thread
        self.processes.write().insert(pid, proc);
        // FIXME: push to ready queue
        self.ready_queue.lock().push_back(pid);
        pid
    }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault
        if err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION){
            return false;
        }
        let now = self.current();
        let pid = now.pid().0 as u64;
        let stack_bot = VirtAddr::new(STACK_MAX - pid * STACK_MAX_SIZE);
        let stack_top = stack_bot - STACK_MAX_SIZE ;
        let apps = self.app_list().unwrap();
        let mut user_access = false;
        for app in apps{
            if now.read().name() == &app.name{
                user_access = true;
            }
        }
        if addr < stack_bot && addr > stack_top{
            if let Ok(_) = now.allocate_stack(stack_bot, addr, user_access){
                return true;
            }
        }
        false
    }

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);
    }

    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  | Status\n");

        for (_, p) in self.processes.read().iter() {
            if p.read().status() != ProgramStatus::Dead {
                output += format!("{}\n", p).as_str();
            }
        }

        // TODO: print memory usage of kernel heap

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }

    pub fn kill_self(&self, ret: isize) {
        self.kill(get_process_manager().current().pid(), ret);
    }
    
}
