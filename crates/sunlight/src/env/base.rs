use std::ffi::c_char;

use super::{CHUNK, load};
use crate::lua::*;
use crate::roblox;

pub unsafe extern "C-unwind" fn loadstring(l: State) -> i32 {
    unsafe { chunk(l, bytes(l, 1), opt_string(l, 2, CHUNK.as_ptr())) }
}

/// Pushes the compiled chunk, or nil and the compile error.
pub unsafe fn chunk(l: State, source: &[u8], name: *const c_char) -> i32 {
    unsafe {
        if load(l, source, name) {
            return 1;
        }
        lua_pushnil(l);
        lua_insert(l, -2);
    }
    2
}

pub unsafe extern "C-unwind" fn getthreadidentity(l: State) -> i32 {
    unsafe { lua_pushinteger(l, roblox::identity(l) as i32) };
    1
}

pub unsafe extern "C-unwind" fn setthreadidentity(l: State) -> i32 {
    unsafe {
        let identity = luaL_checkinteger(l, 1);
        roblox::set_identity(l, identity as u32);
    }
    0
}

pub unsafe extern "C-unwind" fn getreg(l: State) -> i32 {
    unsafe { lua_pushvalue(l, LUA_REGISTRYINDEX) };
    1
}
