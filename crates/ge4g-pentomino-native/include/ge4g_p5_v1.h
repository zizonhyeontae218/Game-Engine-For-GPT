#ifndef GE4G_P5_V1_H
#define GE4G_P5_V1_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Additive P5 ABI1; config v2 and save v2. Does not replace P4 or FlatLand ABI1.
 * Requests borrow len readable bytes; result is owned NUL-terminated UTF-8 JSON.
 * Free each non-null response once with this ABI's string_free.
 * Sessions belong to their creating thread. No callback or concurrent session use.
 * Handles are process-global non-recycled IDs, never Core object capabilities.
 * Max request 24MiB, config 64KiB, trace 4096 operations, 16 sessions/thread.
 */
uint32_t ge4g_p5_v1_abi_version(void);
char *ge4g_p5_v1_request(const uint8_t *data, size_t len);
void ge4g_p5_v1_string_free(char *value);
#ifdef __cplusplus
}
#endif
#endif
