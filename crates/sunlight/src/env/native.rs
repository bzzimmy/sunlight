use std::ffi::CStr;

use super::{base, closure, http, meta};
use crate::lua::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(super) const NATIVES: &[(&CStr, CFunction)] = &[
    (c"newcclosure", closure::newcclosure),
    (c"iscclosure", closure::iscclosure),
    (c"islclosure", closure::islclosure),
    (c"isexecutorclosure", closure::isexecutorclosure),
    (c"clonefunction", closure::clonefunction),
    (c"checkcaller", closure::checkcaller),
    (c"loadstring", base::loadstring),
    (c"getthreadidentity", base::getthreadidentity),
    (c"setthreadidentity", base::setthreadidentity),
    (c"getreg", base::getreg),
    (c"getrawmetatable", meta::getrawmetatable),
    (c"setrawmetatable", meta::setrawmetatable),
    (c"isreadonly", meta::isreadonly),
    (c"setreadonly", meta::setreadonly),
    (c"getnamecallmethod", meta::getnamecallmethod),
    (c"request", http::request),
    (c"httpget", http::httpget),
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
