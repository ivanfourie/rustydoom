# RustyDoom (DoomGeneric in a Rust Shell)

This project hosts [DoomGeneric](https://github.com/ozkl/doomgeneric) inside a Rust application, using:

- [winit](https://crates.io/crates/winit) for cross-platform window/event loop
- [softbuffer](https://crates.io/crates/softbuffer) for simple software framebuffer blitting
- [cc](https://crates.io/crates/cc) to compile DoomGeneric C sources as part of the Rust build

The goal isn’t to make another Doom port, but to **learn Rust by tinkering**:
- Practice building and linking C code from a Rust project
- Experiment with graphics (framebuffer copy, scaling, rendering)
- Extend later with sound, input handling, maybe even a custom renderer
- And, of course, play some Doom while at it 😈

## How it works

- DoomGeneric runs the original Doom logic and produces a 32-bit framebuffer in memory.
- Rust calls into DoomGeneric through FFI (`sys::raw::*`).
- On each frame, the Rust side:
  1. Ticks the Doom engine.
  2. Reads the framebuffer pointer/size.
  3. Copies pixels into a `softbuffer` backbuffer.
  4. Presents the backbuffer into a `winit` window.

That’s it - a minimal Rust shell wrapped around classic Doom.

## Build & Run

You need a Doom IWAD (e.g. `doom1.wad` or `doom2.wad`) in the working directory.

```bash
cargo run --release -- /path/to/doom1.wad
```

Controls are limited (close the window or press Esc to exit).
The project is intentionally minimal so you can experiment and add features as you go.

## Next steps
* Add nearest-neighbor scaling instead of black bars.
* Hook up sound (cpal, rodio, or another Rust audio crate).
* Map input events from winit into DoomGeneric.
* Try replacing softbuffer with wgpu or OpenGL later.
* Use this project as a playground for Rust/C interop.

## Why Doom?
Because Doom is the “hello world” of graphics/game engines — it’s fun, self-contained, and endlessly hackable. Perfect for tinkering with Rust FFI and graphics.