// csrc/platform/i_stubs.c
// No-op audio + joystick backend so we can link without SDL/mixer.

#include <stddef.h>
#include <stdint.h>

// ---- sound layer expects these globals ----
int snd_musicdevice = 0; // 0 = nosound (good enough for linking)

// ---- bind/init/shutdown ----
void I_BindSoundVariables(void) {}
void I_InitSound(void) {}
void I_ShutdownSound(void) {}
void I_InitMusic(void) {}
void I_ShutdownMusic(void) {}
void I_BindJoystickVariables(void) {}
void I_InitJoystick(void) {}

// ---- runtime controls and volumes ----
void I_PauseSound(void) {}
void I_ResumeSound(void) {}
void I_UpdateSound(void) {}
void I_SetMusicVolume(int v) { (void)v; }
void I_SetSfxVolume(int v)   { (void)v; }

// ---- queries/caching ----
int  I_GetSfxLumpNum(void *sfxinfo) { (void)sfxinfo; return -1; } // “not found”
int I_PrecacheSounds(void) {}

// ---- per-sound controls ----
int  I_StartSound(int id, int vol, int sep, int pitch, int priority) {
    (void)id; (void)vol; (void)sep; (void)pitch; (void)priority;
    return -1; // invalid handle
}
void I_StopSound(int handle)                 { (void)handle; }
int  I_SoundIsPlaying(int handle)            { (void)handle; return 0; }
void I_UpdateSoundParams(int handle, int vol, int sep, int pitch) {
    (void)handle; (void)vol; (void)sep; (void)pitch;
}

// ---- music controls ----
void* I_RegisterSong(const void *data, int len) { (void)data; (void)len; return NULL; }
void  I_UnRegisterSong(void *handle)            { (void)handle; }
void  I_PlaySong(void *handle, int looping)     { (void)handle; (void)looping; }
void  I_PauseSong(void *handle)                 { (void)handle; }
void  I_ResumeSong(void *handle)                { (void)handle; }
void  I_StopSong(void *handle)                  { (void)handle; }
