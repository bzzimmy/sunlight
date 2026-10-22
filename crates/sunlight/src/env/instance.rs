use crate::lua::*;
use crate::offsets::*;
use crate::{read, rebase, roblox};

pub unsafe extern "C-unwind" fn instancecache(l: State) -> i32 {
    let key = rebase(INSTANCE_CACHE) as *mut u8;
    unsafe { lua_rawgetptagged(l, LUA_REGISTRYINDEX, key, 0) };
    1
}

/// Pushes the Instance pointer the cache stores its userdata under.
pub unsafe extern "C-unwind" fn instancekey(l: State) -> i32 {
    unsafe { lua_pushlightuserdatatagged(l, read(instance(l), 0), 0) };
    1
}

/// Pushes the cached userdata for the Instance, or a new one if uncached.
pub unsafe extern "C-unwind" fn pushinstance(l: State) -> i32 {
    unsafe { roblox::push_instance(l, instance(l)) };
    1
}

/// Returns the `shared_ptr` held by the Instance userdata argument.
unsafe fn instance(l: State) -> *mut u8 {
    unsafe {
        let tag = read(rebase(INSTANCE_TAG) as *mut u8, 0);
        let instance = lua_touserdatatagged(l, 1, tag);
        if instance.is_null() {
            error(l, c"Instance expected");
        }
        instance
    }
}
