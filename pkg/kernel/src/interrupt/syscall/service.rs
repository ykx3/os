use core::alloc::Layout;

use crate::clock::get_timer_for_sure;
use crate::proc;
use crate::proc::*;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked
    //       - core::slice::from_raw_parts
    // FIXME: spawn the process by name
    // FIXME: handle spawn error, return 0 if failed
    // FIXME: return pid as usize
    let ptr = args.arg0 as *const u8;
    let len = args.arg1 as usize;
    unsafe{
        let buf = core::slice::from_raw_parts(ptr, len);
        let name = core::str::from_utf8_unchecked(&buf);
        if let Some(pid) = spawn(name) {
            pid.0 as usize
        }else{
            0
        }
    }
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    // FIXME: get buffer and fd by args
    //       - core::slice::from_raw_parts
    // FIXME: call proc::write -> isize
    // FIXME: return the result as usize
    let fd = args.arg0 as u8;
    let ptr = args.arg1 as *const u8;
    let len = args.arg2 as usize;
    unsafe{
        let buf = core::slice::from_raw_parts(ptr, len);
        proc::write(fd, buf) as usize
    }
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // FIXME: just like sys_write
    let fd = args.arg0 as u8;
    let ptr = args.arg1 as *mut u8;
    let len = args.arg2 as usize;
    unsafe{
        let mut buf = core::slice::from_raw_parts_mut(ptr, len);
        proc::read(fd, &mut buf) as usize
    }
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcode
    let ret = args.arg0 as isize;
    exit(ret, context)
}

pub fn list_process() {
    // FIXME: list all processes
    print_process_list();
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn sys_time() -> usize {
    let timer = get_timer_for_sure();
    let time = timer.get_time();
    time.nanosecond() as usize / 10 + time.second() as usize * 100 + time.minute() as usize * 60 * 100 + time.hour() as usize * 360000 + time.day() as usize * 24 * 360000
}
