#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

void Rust_Frame(void);

/**
 * Sets up things that could come to be depended on by C code that's called earlier than Rust_Init.
 */
void Rust_Init_Early(void);

void Rust_Init(void);

void CL_Rust_Level_Completed(void);
