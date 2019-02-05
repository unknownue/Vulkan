
use crate::vkchar;

use std::ffi::{ CStr, CString };

/// Helper function to convert [c_char; SIZE] to string
pub fn chars2string(raw_string_array: &[vkchar]) -> String {

    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string.to_str()
        .expect("Failed to convert vulkan raw string to Rust String.")
        .to_owned()
}

pub fn chars2cstring(raw_string_array: &[vkchar]) -> CString {

    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string.to_owned()
}

pub fn cstrings2ptrs(raw_string_array: &[CString]) -> Vec<*const vkchar> {

    raw_string_array.iter()
        .map(|l| l.as_ptr()).collect()
}
