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
