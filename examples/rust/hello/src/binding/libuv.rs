use std::ffi::c_void;

#[allow(non_camel_case_types)]
type uv_loop_t = c_void;

pub mod timer;
pub use timer::UvTimer;
