#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * # Safety
 * Should only be called by Quake during shutdown.
 */
void Rust_Shutdown(void);

void Rust_Frame(void);

/**
 * Sets up things that could come to be depended on by C code that's called earlier than Rust_Init.
 */
void Rust_Init_Early(void);

void Rust_Init(void);

void CL_Rust_Player_Found_Secret(uint16_t secret);

void CL_Rust_Level_Completed(uint16_t skill, const uint16_t *secrets, uintptr_t secrets_len);

void Secret_ClearLocations(void);

void Secret_RecordLocation(uint16_t secret, const vec3_t *mins);

/**
 * Returns -1 if the secret is not found.
 */
int32_t Secret_GetIndexForLocation(const vec3_t *mins);
