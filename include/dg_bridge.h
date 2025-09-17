#pragma once
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

// Minimal creation: builds a tiny argv; if iwad_path != "", passes "-iwad <path>"
int  dg_create_simple(const char* iwad_path);

// Advance one engine tick/frame
void dg_tick(void);

// Expose 32-bit framebuffer (default unless you build with CMAP256)
const uint32_t* dg_framebuffer32(int* w, int* h);

#ifdef __cplusplus
}
#endif
