mod sys;
mod winit_app;

mod constants;
use crate::constants::{APP_NAME, APP_VERSION, INITIAL_WIDTH, INITIAL_HEIGHT};

mod cli;
use cli:: { Cli, normalize_doom_args} ;
use clap::Parser;
use std::env;

mod dg_io;
use dg_io::{ DgIo, DoomHost };

use std::ffi::CString;
use std::num::NonZeroU32;
use std::rc::Rc;
use libc::{c_int, c_uint};

use winit::dpi::LogicalSize;
use winit::window:: {Window, WindowAttributes };
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{EventLoop, ActiveEventLoop};
use winit::keyboard::{Key, NamedKey};

struct State {
    window: Rc<winit::window::Window>,
    context: softbuffer::Context<Rc<winit::window::Window>>,
    io: DgIo,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse_from(normalize_doom_args(env::args()));
    
    // Optional IWAD path; many builds will find it via DOOMWADDIR/cwd.
    let iwad_path = cli.iwad.unwrap_or_default();
    let c_iwad = CString::new(iwad_path).unwrap();
    
    // Boot DoomGeneric and do two warmup ticks.
    // TODO: Create and use an argc, argbv entry point for doomgeneric_Create(argc, argv)
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

impl DoomHost for State {
    fn key_down(&self, _code: i32) {}
    fn key_up(&self, _code: i32) {}
    fn mouse_button(&self, _btn: i32, _down: bool) { println!("mouse_button fired!") }
    fn mouse_move_rel(&self, _dx: f32, _dy: f32) {}
    fn mouse_move_abs(&self, _x: f32, _y: f32) {}
    fn mouse_wheel(&self, _lines: f32) {}
}

struct DgHost;

impl DoomHost for DgHost {
    fn key_down(&self, code: i32)              { unsafe { sys::raw::dg_key_down(code) }; }
    fn key_up(&self, code: i32)                { unsafe { sys::raw::dg_key_up(code) }; }
    fn mouse_button(&self, b: i32, d: bool)    { unsafe { sys::raw::dg_mouse_button(b, if d {1} else {0}) }; }
    fn mouse_move_rel(&self, dx: f32, dy: f32) { unsafe { sys::raw::dg_mouse_move_rel(dx, dy) }; }
    fn mouse_move_abs(&self, x: f32, y: f32)   { unsafe { sys::raw::dg_mouse_move_abs(x, y) }; }
    fn mouse_wheel(&self, lines: f32)          { unsafe { sys::raw::dg_mouse_wheel(lines) }; }
}

fn app_title() -> String {
    format!("{} v{}", APP_NAME, APP_VERSION)
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

/// Nearest-neighbor fit with aspect ratio preserved.
/// dst: window backbuffer (row-major 0x00RRGGBB), size dw*dh
/// src: Doom framebuffer (row-major), size sw*sh
pub fn blit_nn_fit(dst: &mut [u32], dw: usize, dh: usize,
                   src: &[u32], sw: usize, sh: usize) {
    if dw == 0 || dh == 0 || sw == 0 || sh == 0 { return; }

    // Choose target size that fits inside window and preserves aspect.
    // Compare dw/sh vs dh/sw without floats.
    let (tw, th) = if dw * sh <= dh * sw {
        // limited by width
        let tw = dw;
        let th = (dw * sh) / sw;
        (tw, th)
    } else {
        // limited by height
        let th = dh;
        let tw = (dh * sw) / sh;
        (tw, th)
    };

    // Letterbox offsets
    let x0 = (dw - tw) / 2;
    let y0 = (dh - th) / 2;

    // Clear to black
    dst.fill(0x0000_0000);

    // Fixed-point 16.16 stepping for nearest-neighbor
    let x_step = ((sw as u32) << 16) / (tw as u32);
    let y_step = ((sh as u32) << 16) / (th as u32);

    for y in 0..th {
        let sy = ((y as u32 * y_step) >> 16) as usize;
        let src_row = &src[sy * sw .. (sy + 1) * sw];

        let dst_row_start = (y0 + y) * dw + x0;
        let dst_row = &mut dst[dst_row_start .. dst_row_start + tw];

        let mut sx_fp: u32 = 0;
        for dpx in dst_row.iter_mut() {
            let sx = (sx_fp >> 16) as usize;
            *dpx = src_row[sx];
            sx_fp = sx_fp.wrapping_add(x_step);
        }
    }
}


pub(crate) fn entry(event_loop: EventLoop<()>) {
    let app = winit_app::WinitAppBuilder::with_init(
        |elwt| {
           // 1) Create window with an explicit initial size (logical, DPI-aware)
            let window = winit_app::make_window(elwt, |attrs: WindowAttributes| {
                attrs
                    .with_title(app_title())
                    .with_inner_size(LogicalSize::new(
                            INITIAL_WIDTH as f64,
                            INITIAL_HEIGHT as f64,
                    ))
            });
            
            // 2) Create softbuffer context
            let context = softbuffer::Context::new(window.clone()).unwrap();
            
            //(window, context)
            State { window, context, io: DgIo::new() }
        },
        // 3) Create the surface AND perform an initial resize once
       //|_elwt, (window, context)| softbuffer::Surface::new(context, window.clone()).unwrap(),
       // Surface borrows the Context. Pass a reference.
    |_, st: &mut State| {
            softbuffer::Surface::new(&st.context, st.window.clone()).unwrap()
        },
    )
    //.with_event_handler(|(window, _context), surface, event, elwt| {
    .with_event_handler(
        |state: &mut State, 
        surface: Option<&mut softbuffer::Surface<Rc<Window>, Rc<Window>>>, 
        event: Event<()>, 
        elwt: &ActiveEventLoop| {
        let host = DgHost;                    // separate, stateless
        state.io.handle(&host, &state.window, &event);

        // Keep the loop simple: block until events, we’ll request redraws when ready.
        //elwt.set_control_flow(ControlFlow::Wait);
        
        match event {
            // Keep surface size in sync.
            Event::WindowEvent { window_id, event: WindowEvent::Resized(size) }
                if window_id == state.window.id() =>
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
                if window_id == state.window.id() =>
            {
                let Some(surface) = surface else {
                    eprintln!("RedrawRequested fired before Resumed or after Suspended");
                    return;
                };

                // 1) Advance one tic.
                unsafe {
                    sys::raw::dg_pump();  // flush input into Doom
                    sys::raw::dg_tick();  // advance one tic
                }

                // 2) Get Doom’s framebuffer for this tic.
                let Some((fb, fb_w, fb_h)) = fetch_doom_fb() else { return; };

                // 3) Map the backbuffer sized to the current window.
                let size = state.window.inner_size();
                let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return; };
                // Note: we already resize on Resized; no need to call here unless you prefer to.
                let mut backbuf = surface.buffer_mut().unwrap();
                let dst_w = w.get() as usize;
                let dst_h = h.get() as usize;
                let dst: &mut [u32] = &mut backbuf;

                // 4) Scale + letterbox into the backbuffer
                blit_nn_fit(dst, dst_w, dst_h, fb, fb_w, fb_h);

                // 5) Present, then request another redraw to keep things animating.
                backbuf.present().unwrap();
                state.window.request_redraw();
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
            } if window_id == state.window.id() => {
                elwt.exit();
            }

            _ => {}
        }
    });

    winit_app::run_app(event_loop, app);
}
