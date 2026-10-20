use std::ffi::CStr;
use std::ptr;

use super::base;
use crate::lua::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const NATIVES: &[(&CStr, CFunction)] = &[
    (c"newcclosure", newcclosure),
    (c"loadstring", base::loadstring),
    (c"getthreadidentity", base::getthreadidentity),
    (c"setthreadidentity", base::setthreadidentity),
];

/// Pushes the table `env.luau` receives the native functions in.
pub unsafe fn push(l: State) {
    unsafe {
        lua_createtable(l, 0, NATIVES.len() as i32 + 1);
        for (name, function) in NATIVES {
            lua_pushcclosurek(l, *function, name.as_ptr(), 0, None);
            lua_setfield(l, -2, name.as_ptr());
        }
        lua_pushlstring(l, VERSION.as_ptr().cast(), VERSION.len());
        lua_setfield(l, -2, c"version".as_ptr());
    }
}

unsafe extern "C-unwind" fn newcclosure(l: State) -> i32 {
    unsafe {
        luaL_checktype(l, 1, LUA_TFUNCTION);
        let name = opt_string(l, 2, ptr::null());
        lua_pushvalue(l, 1);
        lua_pushcclosurek(l, call, name, 1, Some(finish));
    }
    1
}

unsafe extern "C-unwind" fn call(l: State) -> i32 {
    unsafe {
        let arguments = lua_gettop(l);
        lua_pushvalue(l, lua_upvalueindex(1));
        lua_insert(l, 1);
        lua_call(l, arguments, LUA_MULTRET);
        if lua_status(l) == LUA_OK { lua_gettop(l) } else { -1 }
    }
}

unsafe extern "C-unwind" fn finish(l: State, _status: i32) -> i32 {
    unsafe { lua_gettop(l) }
}
