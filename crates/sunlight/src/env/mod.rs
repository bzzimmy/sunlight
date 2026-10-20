mod base;
mod native;

use std::ffi::{CStr, c_char};

use luau::compile;

use crate::lua::*;
use crate::roblox;

const KEY: &CStr = c"sunlight";
const CHUNK: &CStr = c"=sunlight";

/// Runs `source` on its own thread, creating the shared environment first if
/// this DataModel does not have one yet.
pub unsafe fn execute(script_context: *mut u8, source: &[u8]) {
    unsafe {
        let state = roblox::global_state(script_context);
        let fresh =
            lua_getfield(state, LUA_REGISTRYINDEX, KEY.as_ptr()) == LUA_TNIL;
        lua_settop(state, -2);
        if fresh {
            create(state);
        }
        spawn(state, source, |_| {});
    }
}

unsafe fn create(state: State) {
    unsafe {
        roblox::sandbox_thread(state);
        lua_setfield(state, LUA_REGISTRYINDEX, KEY.as_ptr());
        spawn(state, include_bytes!("env.luau"), |thread| {
            native::push(thread);
            lua_getfield(thread, LUA_REGISTRYINDEX, KEY.as_ptr());
        });
    }
}

/// Defers `source` on a new thread whose globals fall through to the shared
/// environment. `arguments` pushes the values it receives as `...`.
unsafe fn spawn(state: State, source: &[u8], arguments: impl FnOnce(State)) {
    unsafe {
        // stays on the global stack, anchoring the thread until it is deferred
        let thread = lua_newthread(state);
        lua_getfield(thread, LUA_REGISTRYINDEX, KEY.as_ptr());
        lua_replace(thread, LUA_GLOBALSINDEX);
        roblox::sandbox_thread(thread);
        lua_replace(thread, LUA_GLOBALSINDEX);
        roblox::set_identity(thread, 8);

        if load(thread, source, CHUNK.as_ptr()) {
            arguments(thread);
            roblox::task_defer(thread);
        } else {
            let error = lua_tolstring(thread, -1, &mut 0);
            roblox::print(3, CStr::from_ptr(error));
        }
        lua_settop(state, -2);
    }
}

/// Pushes the compiled chunk, or the error message if it does not compile.
unsafe fn load(l: State, source: &[u8], chunk: *const c_char) -> bool {
    let bytecode = compile(source);
    unsafe {
        let loaded = luau_load(l, chunk, bytecode.as_ptr(), bytecode.len(), 0)
            == LUA_OK;
        if loaded {
            roblox::set_proto_capabilities(l, -1, &roblox::CAPABILITIES);
        }
        loaded
    }
}
