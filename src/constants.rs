
// Doom’s “native” framebuffer is 320×200
pub const DOOM_FB_WIDTH:  u32 = 320;
pub const DOOM_FB_HEIGHT: u32 = 200;
pub const SCALE_FACTOR:   u32 = 3;

pub const INITIAL_WIDTH:  u32 = DOOM_FB_WIDTH * SCALE_FACTOR;
pub const INITIAL_HEIGHT: u32 = DOOM_FB_HEIGHT * SCALE_FACTOR;

pub const APP_NAME: &str    = "Rusty Doom";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
