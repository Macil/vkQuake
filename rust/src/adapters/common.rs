use bitflags::bitflags;

#[macro_export]
macro_rules! quake_print {
    ($($arg:tt)*) => {
        $crate::adapters::console::quake_print(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! quake_println {
    ($($arg:tt)*) => {
        $crate::adapters::console::quake_println(&format!($($arg)*));
    };
}

// TODO most of these aren't relevant to register_cvar, so maybe we should hide them.
bitflags! {
    pub struct CvarFlags: std::os::raw::c_uint {
        /// if set, causes it to be saved to config
        const CVAR_ARCHIVE = 1 << 0;
        /// changes will be broadcasted to all players (q1)
        const CVAR_NOTIFY = 1 << 1;
        /// added to serverinfo will be sent to clients (q1/net_dgrm.c and qwsv)
        const CVAR_SERVERINFO = 1 << 2;
        /// added to userinfo, will be sent to server (qwcl)
        const CVAR_USERINFO = 1 << 3;
        const CVAR_CHANGED = 1 << 4;
        const CVAR_ROM = 1 << 6;
        /// locked temporarily
        const CVAR_LOCKED = 1 << 8;
        /// the var is added to the list of variables
        const CVAR_REGISTERED = 1 << 10;
        /// var has a callback
        const CVAR_CALLBACK = 1 << 16;
        /// cvar was created by the user/mod, and needs to be saved a bit differently.
        const CVAR_USERDEFINED = 1 << 17;
        /// cvar changes need to feed back to qc global changes.
        const CVAR_AUTOCVAR = 1 << 18;
    }
}
