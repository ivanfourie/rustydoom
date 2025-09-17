mod sys;
mod winit_app;

use std::ffi::CString;
use std::num::NonZeroU32;
use libc::{c_int, c_uint};

use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};

fn main() -> anyhow::Result<()> {
    // Optional IWAD path; many builds will find it via DOOMWADDIR/cwd.
    let iwad = std::env::args().nth(1).unwrap_or_default();
    let c_iwad = CString::new(iwad).unwrap();

    // Boot DoomGeneric and do two warmup ticks.
    let rc = unsafe { sys::raw::dg_create_simple(c_iwad.as_ptr()) };
    if rc != 0 {
        anyhow::bail!("dg_create_simple failed: {}", rc);
    }
    unsafe { sys::raw::dg_tick() };
    unsafe { sys::raw::dg_tick() };

    // Quick sanity check: read the framebuffer once and print some pixels.
    {
        let (fb, w, h) = fetch_doom_fb().expect("doom framebuffer");
        println!(
            "OK: framebuffer {}x{}, first=0x{:08X}, mid=0x{:08X}",
            w,
            h,
            fb[0],
            fb[(w * h) / 2]
        );
    }

    // Create window + run the app.
    entry(EventLoop::new().unwrap());
    Ok(())
}

/// SAFETY NOTE:
/// DoomGeneric provides a 32-bit framebuffer pointer with out-params for width/height.
/// We wrap that in a slice. This is sound as long as Doom keeps the buffer alive for the frame.
fn fetch_doom_fb() -> Option<(&'static [u32], usize, usize)> {
    let mut w: c_int = 0;
    let mut h: c_int = 0;
    let ptr: *const c_uint = unsafe { sys::raw::dg_framebuffer32(&mut w, &mut h) };
    if ptr.is_null() || w <= 0 || h <= 0 {
        return None;
    }
    let w = w as usize;
    let h = h as usize;
    let len = w * h;
    // SAFETY: ptr points to at least w*h u32 pixels for the duration of this frame.
    let fb = unsafe { std::slice::from_raw_parts(ptr, len) };
    Some((fb, w, h))
}

/// Copy `src` into `dst` centered, without scaling (letterbox).
/// Both are linear row-major u32 buffers.
fn blit_center(dst: &mut [u32], dst_w: usize, dst_h: usize, src: &[u32], src_w: usize, src_h: usize) {
    let copy_w = dst_w.min(src_w);
    let copy_h = dst_h.min(src_h);
    let x_off = (dst_w - copy_w) / 2;
    let y_off = (dst_h - copy_h) / 2;

    // Optional format fixup: set to true if Doom’s pixels aren’t 0x00RRGGBB.
    const NEED_SWIZZLE: bool = false;

    #[inline]
    fn swizzle(px: u32) -> u32 {
        if NEED_SWIZZLE {
            // Example ARGB -> 0x00RRGGBB
            let r = (px >> 16) & 0xFF;
            let g = (px >> 8) & 0xFF;
            let b = px & 0xFF;
            (r << 16) | (g << 8) | b
        } else {
            px
        }
    }

    for y in 0..copy_h {
        let src_row = &src[y * src_w .. y * src_w + copy_w];
        let dst_row_start = (y_off + y) * dst_w + x_off;
        let dst_row = &mut dst[dst_row_start .. dst_row_start + copy_w];

        // Fast path if formats match:
        if !NEED_SWIZZLE {
            dst_row.copy_from_slice(src_row);
        } else {
            for (d, &s) in dst_row.iter_mut().zip(src_row.iter()) {
                *d = swizzle(s);
            }
        }
    }
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
        // Keep the loop simple: block until events, we’ll request redraws when ready.
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            // Keep softbuffer in sync with window size.
            Event::WindowEvent { window_id, event: WindowEvent::Resized(size) }
                if window_id == window.id() =>
            {
                let Some(surface) = surface else {
                    eprintln!("Resized fired before Resumed or after Suspended");
                    return;
                };
                if let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                    surface.resize(w, h).unwrap();
                }
            }

            // One frame: tick Doom -> fetch fb -> map buffer -> clear -> blit -> present -> request next redraw.
            Event::WindowEvent { window_id, event: WindowEvent::RedrawRequested }
                if window_id == window.id() =>
            {
                let Some(surface) = surface else {
                    eprintln!("RedrawRequested fired before Resumed or after Suspended");
                    return;
                };

                // 1) Advance game one tic.
                unsafe { sys::raw::dg_tick() };

                // 2) Get Doom’s framebuffer for this tic.
                let Some((fb, fb_w, fb_h)) = fetch_doom_fb() else { return; };

                // 3) Map the softbuffer backbuffer sized to the current window.
                let size = window.inner_size();
                let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return; };
                // Note: we already resize on Resized; no need to call here unless you prefer to.
                let mut backbuf = surface.buffer_mut().unwrap();
                let dst_w = w.get() as usize;
                let dst_h = h.get() as usize;
                let dst: &mut [u32] = &mut backbuf;

                // 4) Clear to black, then copy Doom’s image centered (no scaling yet).
                dst.fill(0x0000_0000);
                blit_center(dst, dst_w, dst_h, fb, fb_w, fb_h);

                // 5) Present, then request another redraw to keep things animating.
                backbuf.present().unwrap();
                window.request_redraw();
            }

            // Escape or close to quit cleanly.
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested |
                    WindowEvent::KeyboardInput {
                        event: KeyEvent { logical_key: Key::Named(NamedKey::Escape), .. },
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
