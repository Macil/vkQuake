#![allow(clippy::all)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]
// Work around "warning: `extern` block uses type `u128`, which is not FFI-safe" on some platforms.
// https://github.com/rust-lang/rust-bindgen/issues/1549
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
