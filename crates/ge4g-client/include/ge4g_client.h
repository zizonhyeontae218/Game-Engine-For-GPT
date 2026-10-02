#ifndef GE4G_CLIENT_H
#define GE4G_CLIENT_H
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif
uint32_t ge4g_abi_version(void);
char *ge4g_request_json(const char *request);
void ge4g_free_string(char *string);
int64_t ge4g_frame_copy(uint64_t session, uint8_t *destination, size_t capacity);
#ifdef __cplusplus
}
#endif
#endif
