use std::ffi::{CStr, CString, c_char};
use std::fmt::Display;
use std::{ptr, slice};

use crate::offsets::*;
use crate::read;

pub type State = *mut u8;
pub type CFunction = unsafe extern "C-unwind" fn(State) -> i32;
pub type Continuation = unsafe extern "C-unwind" fn(State, i32) -> i32;

pub const LUA_REGISTRYINDEX: i32 = -10000;
pub const LUA_GLOBALSINDEX: i32 = -10002;
pub const LUA_MULTRET: i32 = -1;
pub const LUA_OK: i32 = 0;
pub const LUA_TNIL: i32 = 0;
pub const LUA_TBOOLEAN: i32 = 1;
pub const LUA_TSTRING: i32 = 6;
pub const LUA_TTABLE: i32 = 7;
pub const LUA_TFUNCTION: i32 = 8;

pub const fn lua_upvalueindex(index: i32) -> i32 {
    LUA_GLOBALSINDEX - index
}

/// Returns the string at `index`, or `default` if the argument is absent.
pub unsafe fn opt_string(
    l: State,
    index: i32,
    default: *const c_char,
) -> *const c_char {
    unsafe {
        if lua_type(l, index) > LUA_TNIL {
            luaL_checklstring(l, index, std::ptr::null_mut())
        } else {
            default
        }
    }
}

/// Raises `message` as a Lua error.
pub unsafe fn error(l: State, message: &CStr) -> ! {
    unsafe {
        let error = std::mem::transmute::<
            usize,
            unsafe extern "C-unwind" fn(State, *const c_char, ...) -> i32,
        >(crate::rebase(LUAL_ERROR));
        error(l, c"%s".as_ptr(), message.as_ptr());
    }
    unreachable!()
}

/// Raises a formatted value, such as an `io::Error`, as a Lua error.
pub unsafe fn fail(l: State, message: impl Display) -> ! {
    let message = CString::new(message.to_string()).unwrap_or_default();
    unsafe { error(l, &message) }
}

/// Borrows the string argument at `index`.
pub unsafe fn bytes<'a>(l: State, index: i32) -> &'a [u8] {
    unsafe {
        let mut size = 0;
        let data = luaL_checklstring(l, index, &mut size);
        slice::from_raw_parts(data.cast(), size)
    }
}

pub unsafe fn push_bytes(l: State, bytes: &[u8]) {
    unsafe { lua_pushlstring(l, bytes.as_ptr().cast(), bytes.len()) }
}

/// Returns the collectable object at `index`.
pub unsafe fn pointer(l: State, index: i32) -> *mut u8 {
    unsafe {
        lua_pushvalue(l, index);
        let value =
            read::<*mut u8>(l, THREAD_TOP).sub(16).cast::<*mut u8>().read();
        lua_settop(l, -2);
        value
    }
}

/// Pushes a copy of the value stored at `value`.
pub unsafe fn push(l: State, value: *const u8) {
    unsafe {
        let top: *mut u8 = read(l, THREAD_TOP);
        ptr::copy_nonoverlapping(value, top, 16);
        l.add(THREAD_TOP).cast::<*mut u8>().write(top.add(16));
    }
}

/// Declares wrappers that call `RobloxLib` functions at unslid addresses.
macro_rules! engine {
    ($(fn $name:ident($($arg:ident: $ty:ty),*) $(-> $ret:ty)? = $offset:expr;)*) => {$(
        #[allow(non_snake_case)]
        pub unsafe fn $name($($arg: $ty),*) $(-> $ret)? {
            unsafe {
                std::mem::transmute::<
                    usize,
                    unsafe extern "C-unwind" fn($($ty),*) $(-> $ret)?,
                >(crate::rebase($offset))($($arg),*)
            }
        }
    )*};
}
pub(crate) use engine;

engine! {
    fn lua_newthread(l: State) -> State = LUA_NEWTHREAD;
    fn lua_gettop(l: State) -> i32 = LUA_GETTOP;
    fn lua_settop(l: State, index: i32) = LUA_SETTOP;
    fn lua_insert(l: State, index: i32) = LUA_INSERT;
    fn lua_replace(l: State, index: i32) = LUA_REPLACE;
    fn lua_pushvalue(l: State, index: i32) = LUA_PUSHVALUE;
    fn lua_type(l: State, index: i32) -> i32 = LUA_TYPE;
    fn lua_rawequal(l: State, a: i32, b: i32) -> i32 = LUA_RAWEQUAL;
    fn lua_toboolean(l: State, index: i32) -> bool = LUA_TOBOOLEAN;
    fn lua_tolstring(l: State, index: i32, size: *mut usize) -> *const c_char
        = LUA_TOLSTRING;
    fn lua_tolstringatom(
        l: State,
        index: i32,
        size: *mut usize,
        atom: *mut i32
    ) -> *const c_char = LUA_TOLSTRINGATOM;
    fn lua_namecallatom(l: State, atom: *mut i32) -> *const c_char
        = LUA_NAMECALLATOM;
    fn lua_touserdatatagged(l: State, index: i32, tag: i32) -> *mut u8
        = LUA_TOUSERDATATAGGED;
    fn lua_tothread(l: State, index: i32) -> State = LUA_TOTHREAD;
    fn lua_pushnil(l: State) = LUA_PUSHNIL;
    fn lua_pushboolean(l: State, value: i32) = LUA_PUSHBOOLEAN;
    fn lua_pushlightuserdatatagged(l: State, value: *mut u8, tag: i32)
        = LUA_PUSHLIGHTUSERDATATAGGED;
    fn lua_pushthread(l: State) -> i32 = LUA_PUSHTHREAD;
    fn lua_pushinteger(l: State, value: i32) = LUA_PUSHINTEGER;
    fn lua_pushlstring(l: State, value: *const c_char, size: usize)
        = LUA_PUSHLSTRING;
    fn lua_pushcclosurek(
        l: State,
        function: CFunction,
        name: *const c_char,
        upvalues: i32,
        continuation: Option<Continuation>
    ) = LUA_PUSHCCLOSUREK;
    fn lua_getfield(l: State, index: i32, key: *const c_char) -> i32
        = LUA_GETFIELD;
    fn lua_rawgeti(l: State, index: i32, n: i32) -> i32 = LUA_RAWGETI;
    fn lua_rawgetptagged(l: State, index: i32, key: *mut u8, tag: i32) -> i32
        = LUA_RAWGETPTAGGED;
    fn lua_createtable(l: State, array: i32, records: i32) = LUA_CREATETABLE;
    fn lua_setreadonly(l: State, index: i32, enabled: i32) = LUA_SETREADONLY;
    fn lua_getreadonly(l: State, index: i32) -> i32 = LUA_GETREADONLY;
    fn lua_getmetatable(l: State, index: i32) -> i32 = LUA_GETMETATABLE;
    fn lua_getfenv(l: State, index: i32) = LUA_GETFENV;
    fn lua_setfield(l: State, index: i32, key: *const c_char) = LUA_SETFIELD;
    fn lua_rawset(l: State, index: i32) = LUA_RAWSET;
    fn lua_setmetatable(l: State, index: i32) -> i32 = LUA_SETMETATABLE;
    fn lua_setfenv(l: State, index: i32) -> i32 = LUA_SETFENV;
    fn lua_call(l: State, arguments: i32, results: i32) = LUA_CALL;
    fn lua_pcall(l: State, arguments: i32, results: i32, handler: i32) -> i32
        = LUA_PCALL;
    fn lua_status(l: State) -> i32 = LUA_STATUS;
    fn lua_next(l: State, index: i32) -> i32 = LUA_NEXT;
    fn lua_ref(l: State, index: i32) -> i32 = LUA_REF;
    fn lua_unref(l: State, reference: i32) = LUA_UNREF;
    fn lua_clonefunction(l: State, index: i32) = LUA_CLONEFUNCTION;
    fn lua_yield(l: State, results: i32) -> i32 = LUA_YIELD;
    fn luaL_checklstring(l: State, index: i32, size: *mut usize)
        -> *const c_char = LUAL_CHECKLSTRING;
    fn luaL_checktype(l: State, index: i32, kind: i32) = LUAL_CHECKTYPE;
    fn luaL_checkany(l: State, index: i32) = LUAL_CHECKANY;
    fn luaL_checkinteger(l: State, index: i32) -> i32 = LUAL_CHECKINTEGER;
    fn luau_load(
        l: State,
        chunk: *const c_char,
        data: *const u8,
        size: usize,
        env: i32
    ) -> i32 = LUAU_LOAD;
}
