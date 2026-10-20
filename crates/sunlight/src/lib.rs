mod compile;
mod engine;
mod hooks;
mod image;
mod ipc;
mod offsets;

use std::ffi::c_void;
use std::sync::OnceLock;

use anyhow::{Context, Result};

use engine::{Engine, Step};

static ENGINE: OnceLock<Engine> = OnceLock::new();

#[used]
#[unsafe(link_section = "__DATA,__mod_init_func")]
static INIT: extern "C" fn() = init;

extern "C" fn init() {
    if let Err(error) = attach() {
        eprintln!("sunlight: {error:#}");
    }
}

fn attach() -> Result<()> {
    let slide = image::slide("/RobloxLib.framework/RobloxLib")
        .context("RobloxLib is not loaded")?;
    let engine = ENGINE.get_or_init(|| unsafe { Engine::new(slide) });
    unsafe {
        hooks::write(
            (slide + offsets::SIGGUARD_CALL) as *mut u32,
            offsets::SIGGUARD_CALL_WORD,
            offsets::SIGGUARD_PASS_WORD,
            hooks::READ_EXECUTE,
        )
        .context("failed to bypass SigGuard")?;
        hooks::write(
            (slide + offsets::WHSJ_VTABLE + offsets::STEP_SLOT * 8)
                as *mut usize,
            engine.step as usize,
            step as Step as usize,
            hooks::READ,
        )
    }
    .context("failed to hook WaitingHybridScriptsJob")?;
    ipc::listen()
        .with_context(|| format!("failed to listen on {}", ipc::SOCKET))
}

unsafe extern "C" fn step(job: *mut u8, stats: *mut c_void) -> i32 {
    let engine = ENGINE.get().unwrap();
    let result = unsafe { (engine.step)(job, stats) };
    let data_model: *mut u8 = unsafe { read(job, offsets::JOB_DATA_MODEL) };
    if unsafe { in_game(data_model) } {
        for source in ipc::take() {
            unsafe { engine.execute(job, &source) };
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

unsafe fn read<T>(base: *mut u8, offset: usize) -> T {
    unsafe { base.add(offset).cast::<T>().read() }
}
