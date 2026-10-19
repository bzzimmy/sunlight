mod hooks;
mod image;
mod offsets;

use std::ffi::{c_char, c_void};
use std::mem;
use std::sync::{Mutex, OnceLock, PoisonError};

use anyhow::{Context, Result};

type Print = unsafe extern "C" fn(level: i32, format: *const c_char, ...);
type Step = unsafe extern "C" fn(job: *mut u8, stats: *mut c_void) -> i32;

struct Engine {
    print: Print,
    step: Step,
}

static ENGINE: OnceLock<Engine> = OnceLock::new();
static DATA_MODELS: Mutex<Vec<usize>> = Mutex::new(Vec::new());

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
    let data_model =
        unsafe { job.add(offsets::JOB_DATA_MODEL).cast::<usize>().read() };
    if data_model != 0 && first_seen(data_model) {
        eprintln!("sunlight: DataModel {data_model:#x}");
        unsafe { (engine.print)(0, c"hiiii from sunlight".as_ptr()) };
    }
    result
}

fn first_seen(data_model: usize) -> bool {
    let mut seen = DATA_MODELS.lock().unwrap_or_else(PoisonError::into_inner);
    if seen.contains(&data_model) {
        return false;
    }
    seen.push(data_model);
    true
}
