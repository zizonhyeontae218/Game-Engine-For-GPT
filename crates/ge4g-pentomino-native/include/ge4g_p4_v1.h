#ifndef GE4G_P4_V1_H
#define GE4G_P4_V1_H
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Additive contract. These handles/functions are unrelated to FlatLand ABI1.
 * Sessions are thread-local. Request memory is borrowed for this call only.
 * Every result (including errors) is owned UTF-8 JSON; free exactly once. */
uint32_t ge4g_p4_v1_abi_version(void);
char *ge4g_p4_v1_request(const uint8_t *data, size_t len);
void ge4g_p4_v1_string_free(char *value);
#ifdef __cplusplus
}
#endif
#endif
