use boot::BootInfo;
use boot::RuntimeServices;
use boot::Time;

once_mutex!(pub TIMER: UefiRuntime);

pub fn init(boot_info: &'static boot::BootInfo) {
    init_TIMER(unsafe { UefiRuntime::new(boot_info) });
    info!("Timer Initialized.");
}

pub struct UefiRuntime {
    runtime_service: &'static RuntimeServices,
}

impl UefiRuntime {
    pub unsafe fn new(boot_info: &'static BootInfo) -> Self {
        Self {
            runtime_service: boot_info.system_table.runtime_services(),
        }
    }

    pub fn get_time(&self) -> Time {
        self.runtime_service.get_time().unwrap()
    }
}
guard_access_fn!(pub get_timer(TIMER: UefiRuntime));