use std::ffi::c_char;

use crate::offsets::*;

pub type State = *mut u8;
pub type CFunction = unsafe extern "C-unwind" fn(State) -> i32;
pub type Continuation = unsafe extern "C-unwind" fn(State, i32) -> i32;

pub const LUA_REGISTRYINDEX: i32 = -10000;
pub const LUA_GLOBALSINDEX: i32 = -10002;
pub const LUA_MULTRET: i32 = -1;
pub const LUA_OK: i32 = 0;
pub const LUA_TNIL: i32 = 0;
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
    fn lua_tolstring(l: State, index: i32, size: *mut usize) -> *const c_char
        = LUA_TOLSTRING;
    fn lua_pushnil(l: State) = LUA_PUSHNIL;
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
    fn lua_createtable(l: State, array: i32, records: i32) = LUA_CREATETABLE;
    fn lua_setfield(l: State, index: i32, key: *const c_char) = LUA_SETFIELD;
    fn lua_call(l: State, arguments: i32, results: i32) = LUA_CALL;
    fn lua_status(l: State) -> i32 = LUA_STATUS;
    fn luaL_checklstring(l: State, index: i32, size: *mut usize)
        -> *const c_char = LUAL_CHECKLSTRING;
    fn luaL_checktype(l: State, index: i32, kind: i32) = LUAL_CHECKTYPE;
    fn luaL_checkinteger(l: State, index: i32) -> i32 = LUAL_CHECKINTEGER;
    fn luau_load(
        l: State,
        chunk: *const c_char,
        data: *const u8,
        size: usize,
        env: i32
    ) -> i32 = LUAU_LOAD;
}
