// csrc/platform/dg_host_bridge.c
#include <stdint.h>
#include <string.h>

// Include Doom headers (adjust paths as needed for your tree)
#include "doomdef.h"
#include "d_event.h"
#include "doomstat.h"
#include "d_main.h"
#include "doomkeys.h"

// Mouse wheel: map to PageUp/PageDn which exist in your header
#ifndef KEY_MOUSEWHEELUP
  #define KEY_MOUSEWHEELUP   KEY_PGUP
  #define KEY_MOUSEWHEELDOWN KEY_PGDN
#endif

// Sentinels must match Rust
#define DGK_ENTER   1000
#define DGK_ESCAPE  1001
#define DGK_UP      1100
#define DGK_DOWN    1101
#define DGK_LEFT    1102
#define DGK_RIGHT   1103
#define DGK_USE     1200
#define DGK_FIRE    1201

typedef enum {
  HE_NONE=0, HE_KEY, HE_MOUSE_BTN, HE_MOUSE_REL, HE_MOUSE_ABS, HE_WHEEL
} host_ev_kind;

typedef struct {
  host_ev_kind kind;
  int32_t a, b;
  float   f1, f2;
  int     flag; // 0/1
} host_ev;

#ifndef HOST_QSIZE
#define HOST_QSIZE 256
#endif

static host_ev Q[HOST_QSIZE];
static int q_head=0, q_tail=0;

static inline int q_push(host_ev e){
  int n=(q_tail+1)%HOST_QSIZE;
  if(n==q_head) return 0;
  Q[q_tail]=e; q_tail=n; return 1;
}
static inline int q_pop(host_ev *out){
  if(q_head==q_tail) return 0;
  *out=Q[q_head]; q_head=(q_head+1)%HOST_QSIZE; return 1;
}

static int map_host_key_to_doom(int code) {
  // ASCII (space..~): pass through as-is
  if (code >= 32 && code <= 126) {
    // If you *prefer* Space to be the “use” key regardless of config,
    // comment the next two lines and map Space via DGK_USE in Rust instead.
    // if (code == ' ') return KEY_USE;
    return code;
  }
  switch (code) {
    case DGK_ENTER:  return KEY_ENTER;
    case DGK_ESCAPE: return KEY_ESCAPE;
    case DGK_UP:     return KEY_UPARROW;
    case DGK_DOWN:   return KEY_DOWNARROW;
    case DGK_LEFT:   return KEY_LEFTARROW;
    case DGK_RIGHT:  return KEY_RIGHTARROW;
    case DGK_USE:    return KEY_USE;   // Space (optional)
    case DGK_FIRE:   return KEY_FIRE;  // Ctrl (optional)
    default:         return 0;
  }
}

// ---- Exports for Rust (match sys.rs) ----

void dg_key_down(int code){
  host_ev e={.kind=HE_KEY,.a=map_host_key_to_doom(code),.flag=1};
  q_push(e);
}
void dg_key_up(int code){
  host_ev e={.kind=HE_KEY,.a=map_host_key_to_doom(code),.flag=0};
  q_push(e);
}
void dg_mouse_button(int btn, int down){
  host_ev e={.kind=HE_MOUSE_BTN,.a=btn,.flag=down?1:0};
  q_push(e);
}
void dg_mouse_move_rel(float dx, float dy){
  host_ev e={.kind=HE_MOUSE_REL,.f1=dx,.f2=dy};
  q_push(e);
}
void dg_mouse_move_abs(float x, float y){
  host_ev e={.kind=HE_MOUSE_ABS,.f1=x,.f2=y};
  q_push(e);
}
void dg_mouse_wheel(float lines){
  host_ev e={.kind=HE_WHEEL,.f1=lines};
  q_push(e);
}

void dg_pump(void){
  host_ev e;
  while(q_pop(&e)){
    event_t ev;
    memset(&ev,0,sizeof(ev));
    switch(e.kind){
      case HE_KEY:
        ev.type = e.flag ? ev_keydown : ev_keyup;
        ev.data1 = e.a;
        D_PostEvent(&ev);
        break;

      case HE_MOUSE_BTN: {
        // Doom mouse button bitmask: 1=left, 2=right, 4=middle
        int mask = 0;
        if (e.a == 0) mask = 1;
        else if (e.a == 1) mask = 2;
        else if (e.a == 2) mask = 4;
        ev.type = ev_mouse;
        ev.data1 = e.flag ? mask : 0;
        ev.data2 = 0; ev.data3 = 0;
        D_PostEvent(&ev);
        break;
      }

      case HE_MOUSE_REL:
        ev.type = ev_mouse;
        ev.data1 = 0;
        ev.data2 = (int)e.f1; // dx
        ev.data3 = (int)e.f2; // dy
        D_PostEvent(&ev);
        break;

      case HE_MOUSE_ABS:
        // Ignore or translate to relative if you want
        break;

      case HE_WHEEL: {
        int clicks = (int)e.f1;
        int key = clicks > 0 ? KEY_MOUSEWHEELUP : KEY_MOUSEWHEELDOWN;
        for(int i=0;i<(clicks>0?clicks:-clicks);++i){
          ev.type = ev_keydown; ev.data1 = key; D_PostEvent(&ev);
          ev.type = ev_keyup;   ev.data1 = key; D_PostEvent(&ev);
        }
        break;
      }

      default: break;
    }
  }
}
