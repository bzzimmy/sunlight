use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use lz4_flex::block::{self, DecompressError};

use crate::lua::*;

pub unsafe extern "C-unwind" fn base64encode(l: State) -> i32 {
    unsafe { push_bytes(l, STANDARD.encode(bytes(l, 1)).as_bytes()) };
    1
}

pub unsafe extern "C-unwind" fn base64decode(l: State) -> i32 {
    unsafe {
        let data = STANDARD.decode(bytes(l, 1)).unwrap_or_else(|e| fail(l, e));
        push_bytes(l, &data);
    }
    1
}

pub unsafe extern "C-unwind" fn lz4compress(l: State) -> i32 {
    unsafe { push_bytes(l, &block::compress(bytes(l, 1))) };
    1
}

pub unsafe extern "C-unwind" fn lz4decompress(l: State) -> i32 {
    unsafe {
        let data = bytes(l, 1);
        let mut size = if lua_type(l, 2) > LUA_TNIL {
            luaL_checkinteger(l, 2).max(0) as usize
        } else {
            data.len() * 4
        };
        let data = loop {
            match block::decompress(data, size) {
                Ok(data) => break data,
                Err(DecompressError::OutputTooSmall { .. }) => {
                    size = size * 2 + 1
                }
                Err(e) => fail(l, e),
            }
        };
        push_bytes(l, &data);
    }
    1
}
