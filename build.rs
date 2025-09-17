use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=include/dg_bridge.h");
    println!("cargo:rerun-if-changed=csrc/doomgeneric");
    println!("cargo:rerun-if-changed=csrc/platform");

    let mut build = cc::Build::new();
    build.include("include");
    build.include("csrc/doomgeneric");

    // Core engine + doomgeneric (NO SDL / NO audio backends)
    let core_files = [
        // from SRC_DOOM, with .o -> .c, minus SDL+audio
        "dummy.c", "am_map.c", "doomdef.c", "doomstat.c", "dstrings.c",
        "d_event.c", "d_items.c", "d_iwad.c", "d_loop.c", "d_main.c",
        "d_mode.c", "d_net.c", "f_finale.c", "f_wipe.c", "g_game.c",
        "hu_lib.c", "hu_stuff.c", "info.c",
        // keep these generic I_* that don’t bind SDL
        "i_endoom.c", "i_input.c", "i_scale.c", "i_system.c", "i_timer.c", "i_video.c",
        "memio.c", "m_argv.c", "m_bbox.c", "m_cheat.c", "m_config.c",
        "m_controls.c", "m_fixed.c", "m_menu.c", "m_misc.c", "m_random.c",
        "p_ceilng.c", "p_doors.c", "p_enemy.c", "p_floor.c", "p_inter.c",
        "p_lights.c", "p_map.c", "p_maputl.c", "p_mobj.c", "p_plats.c",
        "p_pspr.c", "p_saveg.c", "p_setup.c", "p_sight.c", "p_spec.c",
        "p_switch.c", "p_telept.c", "p_tick.c", "p_user.c",
        "r_bsp.c", "r_data.c", "r_draw.c", "r_main.c", "r_plane.c",
        "r_segs.c", "r_sky.c", "r_things.c",
        "sha1.c", "statdump.c", "st_lib.c", "st_stuff.c",
        "tables.c", "v_video.c", "wi_stuff.c",
        "s_sound.c","sounds.c", 
        "w_checksum.c", "w_file.c", "w_main.c", "w_wad.c", "z_zone.c",
        "w_file_stdc.c",
        // generic glue
        "doomgeneric.c",
    ];

    for f in core_files {
        build.file(PathBuf::from("csrc/doomgeneric").join(f));
    }

    // Rust-host platform shim (no SDL):
    build.file("csrc/platform/doomgeneric_rust.c");
    // Backend stubs: no-op audio/joystick to satisfy I_* symbols
    build.file("csrc/platform/i_stubs.c");

    // C flags
    build.flag_if_supported("-std=c11");
    build.warnings(true);

    // IMPORTANT: do NOT enable sound yet. The upstream Makefile uses -DFEATURE_SOUND.
    // Remove any FEATURE_SOUND define to avoid pulling in SDL_mixer paths.
    // If your vendored code supports a "no sound" define, you can add it here later, e.g.:
    build.define("DISABLE_SOUND", None);
    // or: build.define("DG_NO_SOUND", None);

    build.compile("doomgeneric");
}
