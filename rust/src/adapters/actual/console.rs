use super::raw_bindings::{Con_SafePrintf, Con_SafePrintf2};
use std::ffi::CString;

#[allow(dead_code)]
pub fn quake_print(message: &str) {
    let fmt = CString::new("%s").unwrap();
    let message = CString::new(message).unwrap();
    unsafe {
        Con_SafePrintf(fmt.as_ptr(), message.as_ptr());
    }
}

#[allow(dead_code)]
pub fn quake_print_no_stdout(message: &str) {
    let fmt = CString::new("%s").unwrap();
    let message = CString::new(message).unwrap();
    unsafe {
        Con_SafePrintf2(false, fmt.as_ptr(), message.as_ptr());
    }
}

#[allow(dead_code)]
pub fn quake_println(message: &str) {
    let fmt = CString::new("%s\n").unwrap();
    let message = CString::new(message).unwrap();
    unsafe {
        Con_SafePrintf(fmt.as_ptr(), message.as_ptr());
    }
}
