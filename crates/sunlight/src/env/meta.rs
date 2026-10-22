use std::ffi::CStr;
use std::ptr;

use crate::lua::*;

pub unsafe extern "C-unwind" fn getrawmetatable(l: State) -> i32 {
    unsafe {
        luaL_checkany(l, 1);
        if lua_getmetatable(l, 1) == 0 {
            lua_pushnil(l);
        }
    }
    1
}

pub unsafe extern "C-unwind" fn setrawmetatable(l: State) -> i32 {
    unsafe {
        luaL_checkany(l, 1);
        if lua_type(l, 2) != LUA_TNIL {
            luaL_checktype(l, 2, LUA_TTABLE);
        }
        lua_settop(l, 2);
        lua_setmetatable(l, 1);
    }
    1
}

pub unsafe extern "C-unwind" fn isreadonly(l: State) -> i32 {
    unsafe {
        luaL_checktype(l, 1, LUA_TTABLE);
        lua_pushboolean(l, lua_getreadonly(l, 1));
    }
    1
}

pub unsafe extern "C-unwind" fn setreadonly(l: State) -> i32 {
    unsafe {
        luaL_checktype(l, 1, LUA_TTABLE);
        luaL_checktype(l, 2, LUA_TBOOLEAN);
        let enabled = lua_toboolean(l, 2);
        if !enabled {
            super::game::detach(pointer(l, 1));
        }
        lua_setreadonly(l, 1, enabled as i32);
    }
    0
}

pub unsafe extern "C-unwind" fn getnamecallmethod(l: State) -> i32 {
    unsafe {
        let name = lua_namecallatom(l, ptr::null_mut());
        if name.is_null() {
            lua_pushnil(l);
        } else {
            lua_pushlstring(l, name, CStr::from_ptr(name).count_bytes());
        }
    }
    1
}
