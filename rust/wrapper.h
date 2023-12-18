// Some hacks to ignore some headers that don't work with Bindgen on all platforms.
#if defined(__i386__) || defined(__x86_64__)
#define __IMMINTRIN_H
#define __XMMINTRIN_H
#define __EMMINTRIN_H
#define __PMMINTRIN_H
#define SDL_cpuinfo_h_
#include <mmintrin.h>
#endif

#include "../Quake/quakedef.h"
#include "../Quake/cfgfile.h"
