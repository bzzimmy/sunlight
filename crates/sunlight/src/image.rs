use std::ffi::{CStr, c_char};

unsafe extern "C" {
    fn _dyld_image_count() -> u32;
    fn _dyld_get_image_name(index: u32) -> *const c_char;
    fn _dyld_get_image_vmaddr_slide(index: u32) -> isize;
}

pub fn slide(suffix: &str) -> Option<usize> {
    let count = unsafe { _dyld_image_count() };
    (0..count).find_map(|index| {
        let name = unsafe { _dyld_get_image_name(index) };
        let name = unsafe { CStr::from_ptr(name) };
        name.to_bytes()
            .ends_with(suffix.as_bytes())
            .then(|| unsafe { _dyld_get_image_vmaddr_slide(index) as usize })
    })
}
