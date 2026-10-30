#include "ge4g_p4_v1.h"
#include "ge4g_p5_v1.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    assert(ge4g_p4_v1_abi_version() == 1);
    assert(ge4g_p5_v1_abi_version() == 1);
    const char *request = "{\"version\":1,\"command\":{\"op\":\"observe\",\"session\":0}}";
    char *response = ge4g_p5_v1_request((const uint8_t *)request, strlen(request));
    assert(response && strstr(response, "\"ok\":false"));
    ge4g_p5_v1_string_free(response);
    ge4g_p5_v1_string_free(NULL);
    puts("P4 preserved + P5 additive C ABI linkage PASS");
    return 0;
}
