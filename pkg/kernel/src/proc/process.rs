use super::*;
use crate::memory::*;
use alloc::sync::Weak;
use alloc::vec::Vec;
use spin::*;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::*;
use elf::{map_range, unmap_range};
use alloc::sync::Arc;

#[derive(Clone)]
pub struct Process {
    pid: ProcessId,
    inner: Arc<RwLock<ProcessInner>>,
}

pub struct ProcessInner {
    name: String,
    parent: Option<Weak<Process>>,
    children: Vec<Arc<Process>>,
    ticks_passed: usize,
    status: ProgramStatus,
    exit_code: Option<isize>,
    context: ProcessContext,
    pub(super) page_table: Option<PageTableContext>,
    proc_data: Option<ProcessData>,
}

impl Process {
    #[inline]
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<ProcessInner> {
        self.inner.write()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<ProcessInner> {
        self.inner.read()
    }

    pub fn new(
        name: String,
        parent: Option<Weak<Process>>,
        page_table: PageTableContext,
        proc_data: Option<ProcessData>,
    ) -> Arc<Self> {
        let name = name.to_ascii_lowercase();

        // create context
        let pid = ProcessId::new();

        let inner = ProcessInner {
            name,
            parent,
            status: ProgramStatus::Ready,
            context: ProcessContext::default(),
            ticks_passed: 0,
            exit_code: None,
            children: Vec::new(),
            page_table: Some(page_table),
            proc_data: Some(proc_data.unwrap_or_default()),
        };

        trace!("New process {}#{} created.", &inner.name, pid);

        // create process struct
        Arc::new(Self {
            pid,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn kill(&self, ret: isize) {
        let mut inner = self.inner.write();

        debug!(
            "Killing process {}#{} with ret code: {}",
            inner.name(),
            self.pid,
            ret
        );

        inner.kill(ret);
    }

    pub fn alloc_init_stack(&self, user_access: bool) -> VirtAddr {
        // FIXME: alloc init stack base on self pid
        let pid = self.pid.0;
        let stack_base = STACK_MAX - pid as u64 * STACK_MAX_SIZE;
        // debug!("1");
        let frame_allocator = &mut *get_frame_alloc_for_sure();
        // debug!("2");
        let mut page_table = self.read().page_table.as_ref().unwrap().mapper();
        // debug!("alloc init stack");
        let flag = if user_access {
            PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE
        }else{
            PageTableFlags::empty()
        };
        self.write().set_stack(VirtAddr::new(stack_base), STACK_DEF_PAGE);
        map_range(stack_base, STACK_DEF_PAGE, &mut page_table, frame_allocator, Some(flag));
        VirtAddr::new(stack_base+STACK_DEF_SIZE-8)
    }

    pub fn allocate_stack(&self, stack_bot:VirtAddr, addr:VirtAddr, user_access: bool) -> Result<(),()>{
        let pages = (stack_bot - addr) / PAGE_SIZE + 1;
        // debug!("alloc stack");
        let flag = if user_access {
            PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE
        }else{
            PageTableFlags::empty()
        };
        self.write().set_stack(stack_bot - PAGE_SIZE * pages, pages);
        let frame_allocator = &mut *get_frame_alloc_for_sure();
        let mut page_table = self.read().page_table.as_ref().unwrap().mapper();
        map_range((stack_bot - PAGE_SIZE * pages).as_u64(), pages, &mut page_table, frame_allocator, Some(flag));
        Ok(())
    }
    
    pub fn set_stack(&mut self, start: VirtAddr, size: u64) {
        self.write().proc_data.as_mut().unwrap().set_stack(start, size);
    }
}

impl ProcessInner {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tick(&mut self) {
        self.ticks_passed += 1;
    }

    pub fn status(&self) -> ProgramStatus {
        self.status
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.status = ProgramStatus::Running;
    }

    pub fn exit_code(&self) -> Option<isize> {
        self.exit_code
    }

    pub fn clone_page_table(&self) -> PageTableContext {
        self.page_table.as_ref().unwrap().clone_l4()
    }

    pub fn is_ready(&self) -> bool {
        self.status == ProgramStatus::Ready
    }

    /// Save the process's context
    /// mark the process as ready
    pub(super) fn save(&mut self, context: &ProcessContext) {
        // FIXME: save the process's context
        self.context.save(context)
    }

    /// Restore the process's context
    /// mark the process as running
    pub(super) fn restore(&mut self, context: &mut ProcessContext) {
        // FIXME: restore the process's context
        self.context.restore(context);
        // FIXME: restore the process's page table
        self.page_table.as_ref().unwrap().load()
    }

    pub fn parent(&self) -> Option<Arc<Process>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn kill(&mut self, ret: isize) {
        // FIXME: set exit code
        self.exit_code = Some(ret);
        // FIXME: set status to dead
        self.status = ProgramStatus::Dead;
        // FIXME: take and drop unused resources
        let frame_deallocator = &mut *get_frame_alloc_for_sure();
        let mut page_table = self.page_table.as_ref().unwrap().mapper();
        let sts = self.proc_data.as_ref().unwrap().stack_segment.unwrap();
        let start_address = sts.start.start_address().as_u64();
        let end_address = sts.end.start_address().as_u64();
        let count = (end_address - start_address) / Size4KiB::SIZE;
        unmap_range(start_address, count, &mut page_table, frame_deallocator);
        drop(self.proc_data.take());
    }

    pub fn init_stack(&mut self, entry:VirtAddr, top:VirtAddr){
        self.context.init_stack_frame(entry, top);
    }
}

impl core::ops::Deref for Process {
    type Target = Arc<RwLock<ProcessInner>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Deref for ProcessInner {
    type Target = ProcessData;

    fn deref(&self) -> &Self::Target {
        self.proc_data
            .as_ref()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::ops::DerefMut for ProcessInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.proc_data
            .as_mut()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut f = f.debug_struct("Process");
        f.field("pid", &self.pid);

        let inner = self.inner.read();
        f.field("name", &inner.name);
        f.field("parent", &inner.parent().map(|p| p.pid));
        f.field("status", &inner.status);
        f.field("ticks_passed", &inner.ticks_passed);
        f.field(
            "children",
            &inner.children.iter().map(|c| c.pid.0).collect::<Vec<u16>>(),
        );
        f.field("page_table", &inner.page_table);
        f.field("status", &inner.status);
        f.field("context", &inner.context);
        f.field("stack", &inner.proc_data.as_ref().map(|d| d.stack_segment));
        f.finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        write!(
            f,
            " #{:-3} \t| #{:-3} \t| {:12} \t| {:7} \t| {:?} \t| {:#?}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            inner.status,
            {
                let sts = inner.proc_data.as_ref().unwrap().stack_segment.unwrap();
                let start_address = sts.start.start_address().as_u64();
                let end_address = sts.end.start_address().as_u64();
                (end_address - start_address) / Size4KiB::SIZE
            }
        )?;
        Ok(())
    }
}
