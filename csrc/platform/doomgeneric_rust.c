#include <stdint.h>
#include <time.h>
#include "doomgeneric.h"
#include "dg_bridge.h"

// Provided by engine
extern pixel_t* DG_ScreenBuffer;

// Sizes are macros in this fork
static inline int dg_width(void)  { return DOOMGENERIC_RESX; }
static inline int dg_height(void) { return DOOMGENERIC_RESY; }

/*======================
  Engine-required hooks
  We DEFINE these; engine CALLS them.
======================*/
void DG_Init(void) {
    // nothing needed; Rust hosts the window/present
}

void DG_DrawFrame(void) {
    // no-op: Rust reads DG_ScreenBuffer and presents via softbuffer
}

void DG_SleepMs(uint32_t ms) {
    struct timespec ts;
    ts.tv_sec  = ms / 1000u;
    ts.tv_nsec = (long)(ms % 1000u) * 1000000L;
    nanosleep(&ts, NULL);
}

uint32_t DG_GetTicksMs(void) {
#if defined(CLOCK_MONOTONIC)
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint32_t)(ts.tv_sec * 1000u + ts.tv_nsec / 1000000u);
#else
    static uint32_t t = 0; t += 16; return t;
#endif
}

// Polled key input: return 1 if you filled (pressed,key), else 0
int DG_GetKey(int* pressed, unsigned char* key) {
    (void)pressed; (void)key; // stub for now; wire later
    return 0;
}

void DG_SetWindowTitle(const char * title) {
    (void)title; // Rust controls window title
}

/*======================
  Rust-facing bridge
======================*/
int dg_create_simple(const char* iwad_path) {
    const char* prog = "rustydoom";
    const char* args[4];
    int argc = 1;
    args[0] = prog;
    if (iwad_path && iwad_path[0]) {
        args[argc++] = "-iwad";
        args[argc++] = iwad_path;
    }
    doomgeneric_Create(argc, (char**)args);
    return 0;
}

void dg_tick(void) {
    doomgeneric_Tick();
}

const uint32_t* dg_framebuffer32(int* w, int* h) {
    if (w) *w = dg_width();
    if (h) *h = dg_height();
    return (const uint32_t*)DG_ScreenBuffer;
}
