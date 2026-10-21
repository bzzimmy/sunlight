use std::ffi::{CStr, c_char, c_void};
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

use super::KEY;
use crate::lua::*;
use crate::offsets::*;
use crate::read;

type Access =
    unsafe extern "C-unwind" fn(State, *mut c_void, i32, *mut u16, i32);
type Namecall =
    unsafe extern "C-unwind" fn(State, *mut c_void, i32, *mut u16, i32) -> i32;

static GAME: AtomicUsize = AtomicUsize::new(0);
static ATOM: AtomicI32 = AtomicI32::new(-1);
static INDEX: AtomicUsize = AtomicUsize::new(0);
static NAMECALL: AtomicUsize = AtomicUsize::new(0);
static INDEX_DIRECT: AtomicUsize = AtomicUsize::new(0);
static NAMECALL_DIRECT: AtomicUsize = AtomicUsize::new(0);

/// Hooks both Instance lookups: the `__index` and `__namecall` closures that
/// unknown names reach, and the per-tag dispatch record the VM calls once it
/// has specialised an access to a known member.
pub unsafe fn install(state: State) {
    unsafe {
        lua_getfield(state, LUA_GLOBALSINDEX, c"game".as_ptr());
        let game = top(state);
        GAME.store(game as usize, Ordering::Relaxed);
        let record = read::<usize>(state, THREAD_GLOBAL)
            + GLOBAL_DISPATCH
            + DISPATCH_SIZE * usize::from(read::<u8>(game, USERDATA_TAG));
        lua_pushlstring(state, c"HttpGetAsync".as_ptr(), 12);
        let mut atom = -1;
        lua_tolstringatom(state, -1, ptr::null_mut(), &mut atom);
        ATOM.store(atom, Ordering::Relaxed);
        lua_settop(state, -2);
        lua_getmetatable(state, -1);
        swap(&INDEX, closure(state, c"__index"), index as *const ());
        swap(&NAMECALL, closure(state, c"__namecall"), namecall as *const ());
        swap(
            &INDEX_DIRECT,
            record + DISPATCH_INDEX,
            index_direct as *const (),
        );
        swap(
            &NAMECALL_DIRECT,
            record + DISPATCH_NAMECALL,
            namecall_direct as *const (),
        );
        lua_settop(state, -3);
    }
}

unsafe fn top(l: State) -> *mut u8 {
    unsafe { read::<*mut u8>(l, THREAD_TOP).sub(16).cast::<*mut u8>().read() }
}

unsafe fn closure(l: State, key: &CStr) -> usize {
    unsafe {
        lua_getfield(l, -1, key.as_ptr());
        let closure = top(l) as usize + CLOSURE_FUNCTION;
        lua_settop(l, -2);
        closure
    }
}

unsafe fn swap(original: &AtomicUsize, slot: usize, hook: *const ()) {
    let previous = unsafe { (slot as *mut usize).replace(hook as usize) };
    original.store(previous, Ordering::Relaxed);
}

unsafe fn original<F>(hook: &AtomicUsize) -> F {
    unsafe { mem::transmute_copy(&hook.load(Ordering::Relaxed)) }
}

unsafe extern "C-unwind" fn index(l: State) -> i32 {
    unsafe {
        if wanted(l, key(l)) {
            push_httpget(l);
            return 1;
        }
        original::<CFunction>(&INDEX)(l)
    }
}

unsafe extern "C-unwind" fn namecall(l: State) -> i32 {
    unsafe {
        if wanted(l, lua_namecallatom(l, ptr::null_mut())) {
            return super::http::httpget(l);
        }
        original::<CFunction>(&NAMECALL)(l)
    }
}

unsafe extern "C-unwind" fn index_direct(
    l: State,
    data: *mut c_void,
    atom: i32,
    slot: *mut u16,
    tag: i32,
) {
    unsafe {
        if atom == ATOM.load(Ordering::Relaxed) && ours(l) {
            return push_httpget(l);
        }
        original::<Access>(&INDEX_DIRECT)(l, data, atom, slot, tag)
    }
}

unsafe extern "C-unwind" fn namecall_direct(
    l: State,
    data: *mut c_void,
    atom: i32,
    slot: *mut u16,
    tag: i32,
) -> i32 {
    unsafe {
        if atom == ATOM.load(Ordering::Relaxed) && ours(l) {
            return super::http::httpget(l);
        }
        original::<Namecall>(&NAMECALL_DIRECT)(l, data, atom, slot, tag)
    }
}

unsafe fn key(l: State) -> *const c_char {
    unsafe {
        if lua_type(l, 2) == LUA_TSTRING {
            lua_tolstring(l, 2, &mut 0)
        } else {
            ptr::null()
        }
    }
}

unsafe fn wanted(l: State, name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    let name = unsafe { CStr::from_ptr(name) };
    (name == c"HttpGet" || name == c"HttpGetAsync") && unsafe { ours(l) }
}

unsafe fn push_httpget(l: State) {
    unsafe {
        lua_getfield(l, LUA_REGISTRYINDEX, KEY.as_ptr());
        lua_getfield(l, -1, c"HttpGet".as_ptr());
        lua_replace(l, -2);
    }
}

/// Whether `self` is `game` and the caller's globals chain to our environment.
unsafe fn ours(l: State) -> bool {
    unsafe {
        lua_pushvalue(l, 1);
        let game = top(l) as usize == GAME.load(Ordering::Relaxed);
        lua_settop(l, -2);
        if !game {
            return false;
        }
        lua_pushvalue(l, LUA_GLOBALSINDEX);
        if lua_getmetatable(l, -1) == 0 {
            lua_settop(l, -2);
            return false;
        }
        lua_getfield(l, -1, c"__index".as_ptr());
        lua_getfield(l, LUA_REGISTRYINDEX, KEY.as_ptr());
        let ours = lua_rawequal(l, -1, -2) != 0;
        lua_settop(l, -5);
        ours
    }
}
