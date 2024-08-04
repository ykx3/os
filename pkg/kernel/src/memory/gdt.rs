use lazy_static::lazy_static;
use x86_64::registers::segmentation::Segment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;
pub const CLOCK_INT_IST_INDEX: u16 = 2;
pub const SYSCALL_IST_INDEX: u16 = 3;

pub const IST_SIZES: [usize; 5] = [0x1000, 0x1000, 0x1000, 0x1000, 0x1000];

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // initialize the TSS with the static buffers
        // will be allocated on the bss section when the kernel is load
        //
        // DO NOT MODIFY THE FOLLOWING CODE
        tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = IST_SIZES[0];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() });
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Privilege Stack  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };

        // FIXME: fill tss.interrupt_stack_table with the static stack buffers like above
        // You can use `tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize]`
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[2];//notice
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() });
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Interrupt Stack(Double Fault)  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };

        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[1];//notice
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() });
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Interrupt Stack(Page Fault)  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };

        tss.interrupt_stack_table[CLOCK_INT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[3];//notice
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() });
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Interrupt Stack(Clock interrupt)  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };

        tss.interrupt_stack_table[SYSCALL_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[4];//notice
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() });
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Interrupt Stack(Syscall interrupt)  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };
        // info!("finish gdt");
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, KernelSelectors, UserSelectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let data_selector = gdt.append(Descriptor::kernel_data_segment());
        let user_code_selector = gdt.append(Descriptor::user_code_segment());
        let user_data_selector = gdt.append(Descriptor::user_data_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        (
            gdt,
            KernelSelectors {
                code_selector,
                data_selector,
                tss_selector,
            },
            UserSelectors {
                user_code_selector,
                user_data_selector,
            }
        )
    };
}

#[derive(Debug)]
pub struct KernelSelectors {
    pub code_selector: SegmentSelector,
    pub data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}
#[derive(Debug)]
pub struct UserSelectors {
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, ES, FS, GS, SS};
    use x86_64::instructions::tables::load_tss;
    use x86_64::PrivilegeLevel;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        SS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        ES::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        FS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        GS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        load_tss(GDT.1.tss_selector);
    }

    let mut size = 0;

    for &s in IST_SIZES.iter() {
        size += s;
    }

    let (size, unit) = crate::humanized_size(size as u64);
    info!("Kernel IST Size  : {:>7.*} {}", 3, size, unit);

    info!("GDT Initialized.");
}

pub fn get_selector() -> &'static KernelSelectors {
    &GDT.1
}

pub fn get_user_selector() -> &'static UserSelectors {
    &GDT.2
}
