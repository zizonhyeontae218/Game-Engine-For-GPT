#include "ge4g_p4_v1.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    const char request[] = "{\"version\":1,\"command\":{\"op\":\"observe\",\"session\":0}}";
    assert(ge4g_p4_v1_abi_version() == 1);
    char *response = ge4g_p4_v1_request((const uint8_t *)request, strlen(request));
    assert(response != NULL);
    assert(strstr(response, "\"ok\":false") != NULL);
    assert(strstr(response, "invalid_session") != NULL);
    puts(response);
    ge4g_p4_v1_string_free(response);
    return 0;
}
