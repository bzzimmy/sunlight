use std::slice;

use super::{CHUNK, load};
use crate::lua::*;
use crate::roblox;

pub unsafe extern "C-unwind" fn loadstring(l: State) -> i32 {
    unsafe {
        let mut size = 0;
        let source = luaL_checklstring(l, 1, &mut size);
        let chunk = opt_string(l, 2, CHUNK.as_ptr());
        if load(l, slice::from_raw_parts(source.cast(), size), chunk) {
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
