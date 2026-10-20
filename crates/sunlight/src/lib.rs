mod compile;
mod env;
mod hooks;
mod image;
mod ipc;
mod lua;
mod offsets;
mod roblox;

use std::ffi::c_void;
use std::panic;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};

static SLIDE: AtomicUsize = AtomicUsize::new(0);

#[used]
#[unsafe(link_section = "__DATA,__mod_init_func")]
static INIT: extern "C" fn() = init;

extern "C" fn init() {
    // a panic must never unwind into Roblox's frames
    panic::set_hook(Box::new(|info| {
        eprintln!("sunlight: {info}");
        process::abort();
    }));
    if let Err(error) = attach() {
        eprintln!("sunlight: {error:#}");
    }
}

fn attach() -> Result<()> {
    let slide = image::slide("/RobloxLib.framework/RobloxLib")
        .context("RobloxLib is not loaded")?;
    SLIDE.store(slide, Ordering::Relaxed);
    unsafe {
        hooks::write(
            rebase(offsets::SIGGUARD_CALL) as *mut u32,
            offsets::SIGGUARD_CALL_WORD,
            offsets::SIGGUARD_PASS_WORD,
            hooks::READ_EXECUTE,
        )
        .context("failed to bypass SigGuard")?;
        hooks::write(
            rebase(offsets::WHSJ_VTABLE + offsets::STEP_SLOT * 8)
                as *mut usize,
            rebase(offsets::WHSJ_STEP),
            step as *const () as usize,
            hooks::READ,
        )
    }
    .context("failed to hook WaitingHybridScriptsJob")?;
    ipc::listen()
        .with_context(|| format!("failed to listen on {}", ipc::SOCKET))
}

unsafe extern "C-unwind" fn step(job: *mut u8, stats: *mut c_void) -> i32 {
    let result = unsafe { roblox::whsj_step(job, stats) };
    let data_model: *mut u8 = unsafe { read(job, offsets::JOB_DATA_MODEL) };
    if unsafe { in_game(data_model) } {
        let script_context = unsafe { read(job, offsets::JOB_SCRIPT_CONTEXT) };
        for source in ipc::take() {
            unsafe { env::execute(script_context, &source) };
        }
    }
    result
}

unsafe fn in_game(data_model: *mut u8) -> bool {
    if data_model.is_null() {
        return false;
    }
    let place_id =
        unsafe { read::<i64>(data_model, offsets::DATA_MODEL_PLACE_ID) };
    let loaded = unsafe { read::<u8>(data_model, offsets::DATA_MODEL_LOADED) };
    place_id != 0 && loaded != 0
}

fn rebase(offset: usize) -> usize {
    SLIDE.load(Ordering::Relaxed) + offset
}

unsafe fn read<T>(base: *mut u8, offset: usize) -> T {
    unsafe { base.add(offset).cast::<T>().read() }
}
