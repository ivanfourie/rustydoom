use libc::{c_char, c_int, c_uint};

pub mod raw {
    use super::*;
    unsafe extern "C" {
        pub fn dg_create_simple(iwad_path: *const c_char) -> c_int;
        pub fn dg_tick();
        pub fn dg_framebuffer32(w: *mut c_int, h: *mut c_int) -> *const c_uint;
    }
}
