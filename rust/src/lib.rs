use libc::c_char;
use std::ffi::CStr;

#[no_mangle]
pub extern "C" fn rust_testing(msg: *const c_char) {
    let msg = unsafe {
        assert!(!msg.is_null());
        CStr::from_ptr(msg)
    }
    .to_string_lossy();

    println!("IN RUST: {}\n", msg);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
