use std::ffi::CStr;

use super::{base, closure, encoding, fs, http, meta};
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
    (c"readfile", fs::readfile),
    (c"writefile", fs::writefile),
    (c"appendfile", fs::appendfile),
    (c"isfile", fs::isfile),
    (c"isfolder", fs::isfolder),
    (c"makefolder", fs::makefolder),
    (c"delfile", fs::delfile),
    (c"delfolder", fs::delfolder),
    (c"listfiles", fs::listfiles),
    (c"loadfile", fs::loadfile),
    (c"getcustomasset", fs::getcustomasset),
    (c"base64encode", encoding::base64encode),
    (c"base64decode", encoding::base64decode),
    (c"lz4compress", encoding::lz4compress),
    (c"lz4decompress", encoding::lz4decompress),
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
