// dg_io.rs
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{CursorGrabMode, Fullscreen, Window},
};
use std::rc::Rc;

pub struct DgIo {
    pub is_fullscreen: bool,
    pub mouse_captured: bool,
    pub mods: ModifiersState,
}

impl DgIo {
    pub fn new() -> Self {
        Self { is_fullscreen: false, mouse_captured: false, mods: ModifiersState::empty() }
    }

    // NOTE: take a host now
    pub fn handle(&mut self, host: &impl DoomHost, window: &Rc<Window>, ev: &Event<()>) {
        match ev {
            Event::WindowEvent { event, .. } => self.handle_window(host, window, event),
            Event::DeviceEvent { event, .. } => self.handle_device(host, event),
            _ => {}
        }
    }

    fn handle_window(&mut self, host: &impl DoomHost, window: &Rc<Window>, ev: &WindowEvent) {
        match ev {
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),

            WindowEvent::KeyboardInput { event: KeyEvent { state, physical_key, .. }, .. } => {
                let pressed = *state == ElementState::Pressed;
                let alt_enter = matches!(physical_key, PhysicalKey::Code(KeyCode::Enter)) && self.mods.alt_key();
                let f11       = matches!(physical_key, PhysicalKey::Code(KeyCode::F11));
                #[cfg(target_os = "macos")]
                let cmd_ctrl_f = matches!(physical_key, PhysicalKey::Code(KeyCode::KeyF)) && self.mods.super_key() && self.mods.control_key();
                #[cfg(not(target_os = "macos"))]
                let cmd_ctrl_f = false;

                if pressed && (alt_enter || f11 || cmd_ctrl_f) {
                    self.toggle_fullscreen(window);
                    return;
                }
                if pressed && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape)) {
                    self.set_mouse_capture(window, false);
                }

                let dk = map_key_to_doom(physical_key);
                if pressed { host.key_down(dk); } else { host.key_up(dk); }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if *state == ElementState::Pressed && *button == MouseButton::Left && !self.mouse_captured {
                    self.set_mouse_capture(window, true);
                }
                match (button, state) {
                    (MouseButton::Left,  ElementState::Pressed)  => host.mouse_button(0, true),
                    (MouseButton::Left,  ElementState::Released) => host.mouse_button(0, false),
                    (MouseButton::Right, ElementState::Pressed)  => host.mouse_button(1, true),
                    (MouseButton::Right, ElementState::Released) => host.mouse_button(1, false),
                    (MouseButton::Middle,ElementState::Pressed)  => host.mouse_button(2, true),
                    (MouseButton::Middle,ElementState::Released) => host.mouse_button(2, false),
                    _ => {}
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                host.mouse_move_abs(position.x as f32, position.y as f32);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                host.mouse_wheel(lines);
            }

            WindowEvent::Focused(false) => {
                if self.mouse_captured {
                    self.set_mouse_capture(window, false);
                }
            }

            _ => {}
        }
    }

    fn handle_device(&mut self, host: &impl DoomHost, ev: &DeviceEvent) {
        if !self.mouse_captured { return; }
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = ev {
            host.mouse_move_rel(*dx as f32, *dy as f32);
        }
    }

    fn toggle_fullscreen(&mut self, window: &Rc<Window>) {
        self.is_fullscreen = !self.is_fullscreen;
        if self.is_fullscreen {
            let mon = window.current_monitor();
            window.set_fullscreen(Some(Fullscreen::Borderless(mon)));
        } else {
            window.set_fullscreen(None);
        }
    }

    fn set_mouse_capture(&mut self, window: &Rc<Window>, capture: bool) {
        if capture {
            let _ = window.set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            window.set_cursor_visible(false);
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
        }
        self.mouse_captured = capture;
    }
}

fn map_key_to_doom(pk: &PhysicalKey) -> i32 {
    use KeyCode::*;
    match pk {
        // ASCII letters (WASD): pass ASCII, Doom reads them fine
        PhysicalKey::Code(KeyW) => b'w' as i32,
        PhysicalKey::Code(KeyA) => b'a' as i32,
        PhysicalKey::Code(KeyS) => b's' as i32,
        PhysicalKey::Code(KeyD) => b'd' as i32,

        // optional: map space/ctrl directly to Doom actions via sentinels
        PhysicalKey::Code(Space)    => DGK_USE,   // default DOOM “use” key
        PhysicalKey::Code(ControlLeft) | PhysicalKey::Code(ControlRight)
                                     => DGK_FIRE, // default DOOM “fire” key

        // arrows and specials via sentinels
        PhysicalKey::Code(ArrowUp)    => DGK_UP,
        PhysicalKey::Code(ArrowDown)  => DGK_DOWN,
        PhysicalKey::Code(ArrowLeft)  => DGK_LEFT,
        PhysicalKey::Code(ArrowRight) => DGK_RIGHT,
        PhysicalKey::Code(Enter)      => DGK_ENTER,
        PhysicalKey::Code(Escape)     => DGK_ESCAPE,

        _ => 0,
    }
}

pub trait DoomHost {
    fn key_down(&self, code: i32);
    fn key_up(&self, code: i32);
    fn mouse_button(&self, btn: i32, down: bool);
    fn mouse_move_rel(&self, dx: f32, dy: f32);
    fn mouse_move_abs(&self, x: f32, y: f32);
    fn mouse_wheel(&self, lines: f32);
}

// keycodes…
pub const DG_KEY_UP: i32 = 1;
pub const DG_KEY_DOWN: i32 = 2;
pub const DG_KEY_LEFT: i32 = 3;
pub const DG_KEY_RIGHT: i32 = 4;
pub const DG_KEY_W: i32 = 5;
pub const DG_KEY_A: i32 = 6;
pub const DG_KEY_S: i32 = 7;
pub const DG_KEY_D: i32 = 8;
pub const DG_KEY_SPACE: i32 = 9;
pub const DG_KEY_ENTER: i32 = 10;
pub const DG_KEY_ESCAPE: i32 = 11;
pub const DG_KEY_UNKNOWN: i32 = 0;

// Host sentinels (must match the C bridge)
pub const DGK_ENTER:  i32 = 1000;
pub const DGK_ESCAPE: i32 = 1001;
pub const DGK_UP:     i32 = 1100;
pub const DGK_DOWN:   i32 = 1101;
pub const DGK_LEFT:   i32 = 1102;
pub const DGK_RIGHT:  i32 = 1103;
pub const DGK_USE:    i32 = 1200; // space -> use (optional)
pub const DGK_FIRE:   i32 = 1201; // LCtrl -> fire (optional)

