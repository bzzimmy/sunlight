use std::ptr;

use super::native::NATIVES;
use crate::lua::*;
use crate::offsets::*;
use crate::read;
use crate::roblox;

pub unsafe extern "C-unwind" fn newcclosure(l: State) -> i32 {
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

pub unsafe extern "C-unwind" fn iscclosure(l: State) -> i32 {
    unsafe { lua_pushboolean(l, is_c(check(l)) as i32) };
    1
}

pub unsafe extern "C-unwind" fn islclosure(l: State) -> i32 {
    unsafe { lua_pushboolean(l, !is_c(check(l)) as i32) };
    1
}

pub unsafe extern "C-unwind" fn isexecutorclosure(l: State) -> i32 {
    unsafe {
        let closure = check(l);
        let ours = if is_c(closure) {
            let function: usize = read(closure, CLOSURE_FUNCTION);
            function == call as *const () as usize
                || NATIVES.iter().any(|(_, native)| {
                    *native as *const () as usize == function
                })
        } else {
            let proto: *mut u8 = read(closure, CLOSURE_PROTO);
            ptr::eq(read(proto, PROTO_CAPABILITIES), &roblox::CAPABILITIES)
        };
        lua_pushboolean(l, ours as i32);
    }
    1
}

pub unsafe extern "C-unwind" fn clonefunction(l: State) -> i32 {
    unsafe {
        let closure = check(l);
        if is_c(closure) {
            let upvalues = read::<u8>(closure, CLOSURE_UPVALUES);
            for i in 0..usize::from(upvalues) {
                push(l, closure.add(CLOSURE_UPVALUE + 16 * i));
            }
            lua_pushcclosurek(
                l,
                read(closure, CLOSURE_FUNCTION),
                read(closure, CLOSURE_NAME),
                i32::from(upvalues),
                read(closure, CLOSURE_CONTINUATION),
            );
        } else {
            lua_clonefunction(l, 1);
            lua_getfenv(l, 1);
            lua_setfenv(l, -2);
        }
    }
    1
}

pub unsafe extern "C-unwind" fn checkcaller(l: State) -> i32 {
    unsafe { lua_pushboolean(l, super::ours(l) as i32) };
    1
}

unsafe fn check(l: State) -> *mut u8 {
    unsafe {
        luaL_checktype(l, 1, LUA_TFUNCTION);
        pointer(l, 1)
    }
}

unsafe fn is_c(closure: *mut u8) -> bool {
    unsafe { read::<u8>(closure, CLOSURE_C) != 0 }
}
