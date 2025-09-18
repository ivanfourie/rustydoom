use libc::{c_char, c_int, c_uint, c_float};

pub mod raw {
    use super::*;
    unsafe extern "C" {
        pub fn dg_create_simple(iwad_path: *const c_char) -> c_int;
        pub fn dg_tick();
        pub fn dg_framebuffer32(w: *mut c_int, h: *mut c_int) -> *const c_uint;
        pub fn dg_key_down(code: c_int);
        pub fn dg_key_up(code: c_int);
        pub fn dg_mouse_button(btn: c_int, down: c_int);
        pub fn dg_mouse_move_rel(dx: c_float, dy: c_float);
        pub fn dg_mouse_move_abs(x: c_float, y: c_float);
        pub fn dg_mouse_wheel(lines: c_float);
        /// Flush queued input into Doom’s event system. Call once per frame before dg_tick().
        pub fn dg_pump();
    }
}
