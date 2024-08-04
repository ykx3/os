use alloc::{collections::BTreeMap, sync::Arc};
use spin::RwLock;
use x86_64::structures::paging::{
    page::PageRange,
    Page,
};

use crate::resource::ResourceSet;

use super::*;

#[derive(Debug, Clone)]
pub struct ProcessData {
    // shared data
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,

    // process specific data
    pub(super) stack_segment: Option<PageRange>,

    pub(super) resources: Arc<RwLock<ResourceSet>>,
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            stack_segment: None,
            resources: Arc::new(RwLock::new(ResourceSet::default()))
        }
    }
}

impl ProcessData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env(&self, key: &str) -> Option<String> {
        self.env.read().get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.env.write().insert(key.into(), val.into());
    }

    pub fn set_stack(&mut self, start: VirtAddr, size: u64) {
        let start = Page::containing_address(start);
        self.stack_segment = Some(Page::range(start, start + size));
    }

    pub fn is_on_stack(&self, addr: VirtAddr) -> bool {
        // FIXME: check if the address is on the stack
        if let Some(stack) = self.stack_segment{
            return addr < stack.end.start_address() && addr > stack.start.start_address();
        }
        return false;
    }
    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.resources.read().read(fd, buf)
    }
    
    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        self.resources.read().write(fd, buf)
    }
}
