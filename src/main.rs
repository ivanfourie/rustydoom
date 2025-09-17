mod sys;
mod winit_app;
use std::num::NonZeroU32;
use std::ffi::CString;
use libc::{c_int, c_uint};

use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};

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

    // Create window
    entry(EventLoop::new().unwrap());

    Ok(())
}

pub(crate) fn entry(event_loop: EventLoop<()>) {
    let app = winit_app::WinitAppBuilder::with_init(
        |elwt| {
            let window = winit_app::make_window(elwt, |w| w);

            let context = softbuffer::Context::new(window.clone()).unwrap();

            (window, context)
        },
        |_elwt, (window, context)| softbuffer::Surface::new(context, window.clone()).unwrap(),
    )
    .with_event_handler(|(window, _context), surface, event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(size),
            } if window_id == window.id() => {
                let Some(surface) = surface else {
                    eprintln!("Resized fired before Resumed or after Suspended");
                    return;
                };

                if let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    surface.resize(width, height).unwrap();
                }
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
            } if window_id == window.id() => {
                let Some(surface) = surface else {
                    eprintln!("RedrawRequested fired before Resumed or after Suspended");
                    return;
                };
                
                // 1) advance DG
                unsafe { sys::raw::dg_tick() };

                // 2) get DG framebuffer
                let mut fb_w: c_int = 0;
                let mut fb_h: c_int = 0;
                let ptr: *const c_uint = unsafe { sys::raw::dg_framebuffer32(&mut fb_w, &mut fb_h) };
                if ptr.is_null() { return; }
                let fb_w = fb_w as usize;
                let fb_h = fb_h as usize;
                let fb = unsafe { std::slice::from_raw_parts(ptr, fb_w * fb_h) }; // &[u32]

                // 3) ensure surface size is current
                let size = window.inner_size();
                let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
                    return;
                };
                surface.resize(w, h).unwrap();

                // 4) map buffer and blit
                let mut buf = surface.buffer_mut().unwrap(); // Derefs to &mut [u32]
                let sb_w = w.get() as usize;
                let sb_h = h.get() as usize;
                let dst: &mut [u32] = &mut buf; // coerce to slice

                // Clear to black (0x00_00_00 per softbuffer format)
                dst.fill(0x0000_0000);

                // Simple 1:1 copy centered; add scaling later if you want
                let copy_w = sb_w.min(fb_w);
                let copy_h = sb_h.min(fb_h);
                let x_off = (sb_w - copy_w) / 2;
                let y_off = (sb_h - copy_h) / 2;

                // If your DG pixels are already 0x00RRGGBB, fast path; otherwise enable swizzle() below.
                #[inline]
                fn swizzle(s: u32) -> u32 {
                    // Convert ARGB (0xAARRGGBB) -> 0x00RRGGBB if needed:
                    // let r = (s >> 16) & 0xFF; let g = (s >> 8) & 0xFF; let b = s & 0xFF;
                    // (r << 16) | (g << 8) | b
                    s // no-op if already 0x00RRGGBB
                }

                for y in 0..copy_h {
                    let src_row = &fb[y * fb_w .. y * fb_w + copy_w];
                    let dst_row_start = (y_off + y) * sb_w + x_off;
                    let dst_row = &mut dst[dst_row_start .. dst_row_start + copy_w];

                    // Fast copy if formats match:
                    // dst_row.copy_from_slice(src_row);

                    // Safe copy with optional per-pixel swizzle:
                    for (d, &s) in dst_row.iter_mut().zip(src_row.iter()) {
                        *d = swizzle(s);
                    }
                }

                buf.present().unwrap();

                // keep the loop running
                window.request_redraw();
     

            }
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    },
                window_id,
            } if window_id == window.id() => {
                elwt.exit();
            }
            _ => {}
        }
    });

    winit_app::run_app(event_loop, app);
}