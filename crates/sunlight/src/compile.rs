use std::ffi::{c_char, c_void};
use std::slice;

unsafe extern "C" {
    fn sunlight_compile(
        source: *const c_char,
        size: usize,
        out: *mut usize,
    ) -> *mut c_char;
    fn free(pointer: *mut c_void);
}

/// Compiles to client bytecode. Errors are encoded for `luau_load` to report.
pub fn compile(source: &[u8]) -> Vec<u8> {
    let mut size = 0;
    unsafe {
        let data =
            sunlight_compile(source.as_ptr().cast(), source.len(), &mut size);
        let bytecode = slice::from_raw_parts(data.cast::<u8>(), size).to_vec();
        free(data.cast());
        bytecode
    }
}
