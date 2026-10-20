use std::ffi::{c_char, c_void};
use std::{mem, ptr};

use crate::compile::compile;
use crate::{offsets, read};

type State = *mut u8;
pub type Step = unsafe extern "C" fn(*mut u8, *mut c_void) -> i32;

const LUA_GLOBALSINDEX: i32 = -10002;
static CAPABILITIES: u64 = u64::MAX;

pub struct Engine {
    pub step: Step,
    print: unsafe extern "C" fn(i32, *const c_char, ...),
    global_state: unsafe extern "C" fn(*mut u8, *mut u64, *mut u64) -> State,
    new_thread: unsafe extern "C" fn(State) -> State,
    set_top: unsafe extern "C" fn(State, i32),
    to_string: unsafe extern "C" fn(State, i32, *mut usize) -> *const c_char,
    replace: unsafe extern "C" fn(State, i32),
    sandbox: unsafe extern "C" fn(State),
    load: unsafe extern "C" fn(
        State,
        *const c_char,
        *const u8,
        usize,
        i32,
    ) -> i32,
    set_capabilities: unsafe extern "C" fn(State, i32, *const u64),
    defer: unsafe extern "C" fn(State) -> i32,
}

impl Engine {
    pub unsafe fn new(slide: usize) -> Self {
        unsafe fn at<T>(address: usize) -> T {
            unsafe { mem::transmute_copy(&address) }
        }
        unsafe {
            Self {
                step: at(slide + offsets::WHSJ_STEP),
                print: at(slide + offsets::PRINT),
                global_state: at(slide + offsets::GET_GLOBAL_STATE),
                new_thread: at(slide + offsets::LUA_NEWTHREAD),
                set_top: at(slide + offsets::LUA_SETTOP),
                to_string: at(slide + offsets::LUA_TOLSTRING),
                replace: at(slide + offsets::LUA_REPLACE),
                sandbox: at(slide + offsets::SANDBOX_THREAD),
                load: at(slide + offsets::LUAU_LOAD),
                set_capabilities: at(slide + offsets::SET_PROTO_CAPABILITIES),
                defer: at(slide + offsets::TASK_DEFER),
            }
        }
    }

    pub unsafe fn execute(&self, job: *mut u8, source: &str) {
        let bytecode = compile(source);
        unsafe {
            let script_context = read(job, offsets::JOB_SCRIPT_CONTEXT);
            let state = (self.global_state)(script_context, &mut 0, &mut 0);
            // stays on the global stack, anchoring the thread until it is deferred
            let thread = (self.new_thread)(state);
            (self.sandbox)(thread);
            (self.replace)(thread, LUA_GLOBALSINDEX);

            let data: *mut u8 = read(thread, offsets::THREAD_DATA);
            data.add(offsets::THREAD_IDENTITY).cast::<u32>().write(8);
            data.add(offsets::THREAD_CAPABILITIES)
                .cast::<u64>()
                .write(CAPABILITIES);

            let chunk = c"=sunlight".as_ptr();
            if (self.load)(thread, chunk, bytecode.as_ptr(), bytecode.len(), 0)
                == 0
            {
                (self.set_capabilities)(thread, -1, &CAPABILITIES);
                (self.defer)(thread);
            } else {
                let error = (self.to_string)(thread, -1, ptr::null_mut());
                (self.print)(3, c"%s".as_ptr(), error);
            }
            (self.set_top)(state, -2);
        }
    }
}
