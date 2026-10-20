use std::ffi::c_void;
use std::fmt::LowerHex;

use anyhow::{Result, ensure};

pub const READ: i32 = 0x1;
pub const READ_EXECUTE: i32 = 0x5;
const WRITE_COPY: i32 = 0x12;

unsafe extern "C" {
    static mach_task_self_: u32;
    fn mach_vm_protect(
        task: u32,
        address: u64,
        size: u64,
        max: i32,
        protection: i32,
    ) -> i32;
    fn sys_icache_invalidate(start: *mut c_void, size: usize);
}

/// Overwrites protected memory, failing if it does not hold `expected`.
pub unsafe fn write<T: Copy + PartialEq + LowerHex>(
    target: *mut T,
    expected: T,
    value: T,
    protection: i32,
) -> Result<()> {
    let protect = |protection| unsafe {
        mach_vm_protect(
            mach_task_self_,
            target as u64,
            size_of::<T>() as u64,
            0,
            protection,
        )
    };
    let status = protect(READ | WRITE_COPY);
    ensure!(status == 0, "mach_vm_protect failed with {status}");
    let actual = unsafe { target.read() };
    if actual == expected {
        unsafe {
            target.write(value);
            sys_icache_invalidate(target.cast(), size_of::<T>());
        }
    }
    protect(protection);
    ensure!(
        actual == expected,
        "expected {expected:#x} at {target:?}, found {actual:#x}"
    );
    Ok(())
}
