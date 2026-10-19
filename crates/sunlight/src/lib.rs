mod hooks;
mod image;
mod offsets;

use std::ffi::{c_char, c_void};
use std::mem;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};

type Print = unsafe extern "C" fn(level: i32, format: *const c_char, ...);
type Step = unsafe extern "C" fn(job: *mut u8, stats: *mut c_void) -> i32;

struct Engine {
    print: Print,
    step: Step,
}

static ENGINE: OnceLock<Engine> = OnceLock::new();
static GAME: AtomicUsize = AtomicUsize::new(0);

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
    let engine = ENGINE.get_or_init(|| unsafe {
        Engine {
            print: mem::transmute::<usize, Print>(slide + offsets::PRINT),
            step: mem::transmute::<usize, Step>(slide + offsets::WHSJ_STEP),
        }
    });
    let vtable = (slide + offsets::WHSJ_VTABLE) as *mut usize;
    unsafe {
        hooks::replace(
            vtable.add(offsets::STEP_SLOT),
            engine.step as usize,
            step as Step as usize,
        )
    }
    .context("failed to hook WaitingHybridScriptsJob")
}

unsafe extern "C" fn step(job: *mut u8, stats: *mut c_void) -> i32 {
    let engine = ENGINE.get().unwrap();
    let result = unsafe { (engine.step)(job, stats) };
    let data_model = unsafe { read::<*mut u8>(job, offsets::JOB_DATA_MODEL) };
    if unsafe { joined(data_model) } {
        unsafe { (engine.print)(0, c"hiiii from sunlight".as_ptr()) };
    }
    result
}

unsafe fn joined(data_model: *mut u8) -> bool {
    if data_model.is_null() {
        return false;
    }
    let place_id =
        unsafe { read::<i64>(data_model, offsets::DATA_MODEL_PLACE_ID) };
    place_id != 0
        && GAME.swap(data_model as usize, Ordering::Relaxed)
            != data_model as usize
}

unsafe fn read<T>(base: *mut u8, offset: usize) -> T {
    unsafe { base.add(offset).cast::<T>().read() }
}
