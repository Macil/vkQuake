#![allow(dead_code)]

use super::{
    game::{Game, GameInit},
    raw_bindings::{
        cvar_t, CFG_CloseConfig, CFG_OpenConfig, CFG_ReadCvars, Cvar_FindVar,
        Cvar_RegisterVariable, Cvar_SetQuick, Cvar_SetValueQuick,
    },
};
use anyhow::anyhow;
use std::ffi::CString;

pub use super::super::common::CvarFlags;

#[derive(Debug)]
pub struct Cvar(*mut cvar_t);

impl Cvar {
    // TODO provide ways to subscribe to value changes.
    pub fn register(
        _game_init: &mut GameInit,
        name: &str,
        default: &str,
        flags: CvarFlags,
    ) -> Cvar {
        // leak the values so they can be accessed by the game engine forever.
        let name = Box::leak(Box::new(CString::new(name).unwrap()));
        let default = Box::leak(Box::new(CString::new(default).unwrap()));
        let cvar = Box::leak(Box::new(cvar_t {
            name: name.as_ptr(),
            string: default.as_ptr(),
            flags: flags.bits(),
            value: 0.0,
            default_string: std::ptr::null(),
            callback: None,
            next: std::ptr::null_mut(),
        }));
        unsafe {
            Cvar_RegisterVariable(cvar);
        }
        Cvar(cvar)
    }

    /// Must be used if you want to read the value of a cvar before the game is initialized.
    pub fn load_early(_game_init: &mut GameInit, names: &[&str]) -> anyhow::Result<()> {
        let config_filename = CString::new("config.cfg").unwrap();
        let cnames = names
            .iter()
            .map(|&name| CString::new(name).unwrap())
            .collect::<Vec<_>>();
        let mut cname_ptrs = cnames
            .iter()
            .map(|cname| cname.as_ptr())
            .collect::<Vec<_>>();
        unsafe {
            if CFG_OpenConfig(config_filename.as_ptr()) == 0 {
                CFG_ReadCvars(
                    cname_ptrs.as_mut_ptr(),
                    cname_ptrs.len() as std::os::raw::c_int,
                );
                CFG_CloseConfig();
            }
            Ok(())
        }
    }

    pub fn find_existing_by_name(_game: &Game, name: &str) -> anyhow::Result<Cvar> {
        let cname = CString::new(name).unwrap();
        let cvar = unsafe { Cvar_FindVar(cname.as_ptr()) };
        if cvar.is_null() {
            Err(anyhow!("Failed to find cvar {:?}", name))
        } else {
            Ok(Cvar(cvar))
        }
    }

    pub fn name(&self) -> &str {
        unsafe { std::ffi::CStr::from_ptr((*self.0).name) }
            .to_str()
            .expect("Cvar name is not UTF-8")
    }

    pub fn str_value<'a>(&self, _game: &'a Game) -> Result<Option<&'a str>, std::str::Utf8Error> {
        let s = unsafe { (*self.0).string };
        if s.is_null() {
            Ok(None)
        } else {
            unsafe { std::ffi::CStr::from_ptr(s) }.to_str().map(Some)
        }
    }

    pub fn value(&self, _game: &Game) -> f32 {
        unsafe { (*self.0).value }
    }

    pub fn set_value(&self, _game: &mut Game, value: f32) {
        unsafe {
            Cvar_SetValueQuick(self.0, value);
        }
    }

    pub fn set_str_value(&self, _game: &mut Game, value: &str) {
        let value = CString::new(value).unwrap();
        unsafe {
            Cvar_SetQuick(self.0, value.as_ptr());
        }
    }
}

unsafe impl Send for Cvar {}
unsafe impl Sync for Cvar {}
