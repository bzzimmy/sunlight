use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Result, bail, ensure};

const VM_PROT_READ: i32 = 0x1;
const VM_PROT_WRITE: i32 = 0x2;
const VM_PROT_COPY: i32 = 0x10;

unsafe extern "C" {
    static mach_task_self_: u32;
    fn mach_vm_protect(
        task: u32,
        address: u64,
        size: u64,
        set_maximum: i32,
        protection: i32,
    ) -> i32;
}

/// Swaps a pointer in read-only data, failing if it does not hold `expected`.
pub unsafe fn replace(
    slot: *mut usize,
    expected: usize,
    value: usize,
) -> Result<()> {
    let protect = |protection| unsafe {
        mach_vm_protect(mach_task_self_, slot as u64, 8, 0, protection)
    };
    let status = protect(VM_PROT_READ | VM_PROT_WRITE | VM_PROT_COPY);
    ensure!(status == 0, "mach_vm_protect failed with {status}");
    let swapped = unsafe { AtomicUsize::from_ptr(slot) }.compare_exchange(
        expected,
        value,
        Ordering::Relaxed,
        Ordering::Relaxed,
    );
    protect(VM_PROT_READ);
    if let Err(actual) = swapped {
        bail!("expected {expected:#x} at {slot:?}, found {actual:#x}");
    }
    Ok(())
}
