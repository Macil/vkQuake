#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * # Safety
 * Must only be called by Quake on the main game thread.
 */
void Rust_Frame(void);

void Rust_Init(void);
