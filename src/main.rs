mod sys;
use std::ffi::CString;
use libc::{c_int, c_uint};

fn main() -> anyhow::Result<()> {
    // Optional IWAD path; many builds will find it via DOOMWADDIR/cwd
    let iwad = std::env::args().nth(1).unwrap_or_default();
    let c_iwad = CString::new(iwad).unwrap();

    // Create and tick a couple of frames
    let rc = unsafe { sys::raw::dg_create_simple(c_iwad.as_ptr()) };
    if rc != 0 { anyhow::bail!("dg_create_simple failed: {}", rc); }

    unsafe { sys::raw::dg_tick(); }
    unsafe { sys::raw::dg_tick(); }

    // Fetch framebuffer and print a few values as a sanity check
    let mut w: c_int = 0;
    let mut h: c_int = 0;
    let ptr: *const c_uint = unsafe { sys::raw::dg_framebuffer32(&mut w, &mut h) };
    assert!(!ptr.is_null(), "framebuffer ptr is null");
    let len = (w as usize) * (h as usize);
    let fb = unsafe { std::slice::from_raw_parts(ptr, len) };
    println!("OK: framebuffer {}x{}, first=0x{:08X}, mid=0x{:08X}", w, h, fb[0], fb[len/2]);

    Ok(())
}
