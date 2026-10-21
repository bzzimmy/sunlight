use std::ffi::CStr;
use std::mem;
use std::sync::Mutex;
use std::thread;

use crate::lua::*;
use crate::offsets::*;
use crate::roblox;

/// Pushes the results of a finished job onto a thread and returns their count.
pub type Reply = Box<dyn FnOnce(State) -> i32 + Send>;

struct Pending {
    thread: usize,
    reference: i32,
    reply: Reply,
}

static PENDING: Mutex<Vec<Pending>> = Mutex::new(Vec::new());

/// Yields `l` while `work` runs on its own thread; `resume` wakes it with the
/// reply once the game steps again.
pub unsafe fn suspend(
    l: State,
    work: impl FnOnce() -> Reply + Send + 'static,
) -> i32 {
    unsafe {
        let yielded = lua_yield(l, 0);
        lua_pushthread(l);
        let reference = lua_ref(l, -1);
        lua_settop(l, -2);
        let thread = l as usize;
        thread::spawn(move || {
            let reply = work();
            let pending = Pending { thread, reference, reply };
            PENDING.lock().unwrap().push(pending);
        });
        yielded
    }
}

/// Resumes finished jobs through `task.defer`, skipping threads whose VM died.
pub unsafe fn resume(state: State) {
    let jobs = mem::take(&mut *PENDING.lock().unwrap());
    for job in jobs {
        unsafe {
            let scratch = lua_newthread(state);
            let defer =
                mem::transmute::<usize, CFunction>(crate::rebase(TASK_DEFER));
            lua_pushcclosurek(scratch, defer, c"defer".as_ptr(), 0, None);
            lua_rawgeti(scratch, LUA_REGISTRYINDEX, job.reference);
            if lua_tothread(scratch, -1) as usize == job.thread {
                let results = (job.reply)(scratch);
                if lua_pcall(scratch, results + 1, 0, 0) != LUA_OK {
                    let error = lua_tolstring(scratch, -1, &mut 0);
                    if !error.is_null() {
                        roblox::print(3, CStr::from_ptr(error));
                    }
                }
                lua_unref(scratch, job.reference);
            }
            lua_settop(state, -2);
        }
    }
}
