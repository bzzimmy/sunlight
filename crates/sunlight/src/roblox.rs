use std::ffi::{CStr, c_char, c_void};
use std::mem;

use crate::lua::{State, engine};
use crate::offsets::*;
use crate::read;

pub static CAPABILITIES: u64 = u64::MAX;

engine! {
    fn whsj_step(job: *mut u8, stats: *mut c_void) -> i32 = WHSJ_STEP;
    fn get_global_state(
        script_context: *mut u8,
        identity: *mut u64,
        actor: *mut u64
    ) -> State = GET_GLOBAL_STATE;
    fn sandbox_thread(l: State) = SANDBOX_THREAD;
    fn set_proto_capabilities(l: State, index: i32, capabilities: *const u64)
        = SET_PROTO_CAPABILITIES;
    fn task_defer(l: State) -> i32 = TASK_DEFER;
}

pub unsafe fn print(level: i32, message: &CStr) {
    unsafe {
        let print = mem::transmute::<
            usize,
            unsafe extern "C-unwind" fn(i32, *const c_char, ...),
        >(crate::rebase(PRINT));
        print(level, c"%s".as_ptr(), message.as_ptr());
    }
}

pub unsafe fn global_state(script_context: *mut u8) -> State {
    unsafe { get_global_state(script_context, &mut 0, &mut 0) }
}

pub unsafe fn identity(l: State) -> u32 {
    unsafe { read(read(l, THREAD_DATA), THREAD_IDENTITY) }
}

pub unsafe fn set_identity(l: State, identity: u32) {
    unsafe {
        let data: *mut u8 = read(l, THREAD_DATA);
        data.add(THREAD_IDENTITY).cast::<u32>().write(identity);
        data.add(THREAD_CAPABILITIES).cast::<u64>().write(CAPABILITIES);
    }
}
