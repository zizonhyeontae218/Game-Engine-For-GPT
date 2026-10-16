#include "ge4g_p4_v1.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* One line/request; expect success and return every owned response exactly once. */
int main(int argc, char **argv) {
    assert(argc == 2);
    FILE *f = fopen(argv[1], "rb");
    assert(f != NULL);
    char request[65536];
    size_t calls = 0;
    assert(ge4g_p4_v1_abi_version() == 1);
    ge4g_p4_v1_string_free(NULL);
    while (fgets(request, sizeof(request), f)) {
        char *response = ge4g_p4_v1_request((const uint8_t *)request, strlen(request));
        assert(response != NULL);
        if (strstr(response, "\"ok\":true") == NULL) {
            fputs(response, stderr);
            abort();
        }
        ge4g_p4_v1_string_free(response);
        calls++;
    }
    assert(fclose(f) == 0);
    printf("PASS %zu owned responses freed\n", calls);
    return 0;
}
