use std::ffi::{CStr, c_char, c_void};
use std::mem;

use crate::lua::{State, engine};
use crate::offsets::*;
use crate::read;

pub static CAPABILITIES: u64 = u64::MAX;
const INSTANCE_CAPABILITIES: u64 = 0x3FFFFFFFFFFF00;

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
    fn identity_capabilities(identity: *const u32) -> u64
        = IDENTITY_CAPABILITIES;
    fn security_context() -> *mut u8 = SECURITY_CONTEXT;
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
        let capabilities =
            identity_capabilities(&identity) | INSTANCE_CAPABILITIES;
        data.add(THREAD_CAPABILITIES).cast::<u64>().write(capabilities);
        // the engine checks a per-resume copy of both until the thread yields
        let context = security_context();
        if read::<State>(context, CONTEXT_THREAD) == l {
            context.cast::<u32>().write(identity);
            context
                .add(CONTEXT_CAPABILITIES)
                .cast::<u64>()
                .write(capabilities);
        }
    }
}
