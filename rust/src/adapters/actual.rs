use std::path::PathBuf;

pub mod console;
pub mod game;
mod raw_bindings;

pub fn game_pref_path() -> PathBuf {
    let empty = std::ffi::CString::new("").unwrap();
    let vkquake = std::ffi::CString::new("vkQuake").unwrap();
    unsafe {
        let path_ptr = raw_bindings::SDL_GetPrefPath(empty.as_ptr(), vkquake.as_ptr());
        let pathbuf = std::ffi::CStr::from_ptr(path_ptr).to_str().unwrap().into();
        raw_bindings::SDL_free(path_ptr as *mut std::ffi::c_void);
        pathbuf
    }
}
