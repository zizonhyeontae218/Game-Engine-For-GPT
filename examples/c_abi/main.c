#include <stdio.h>
#include <string.h>
#include "ge4g_client.h"

/* shortcut: one JSON request per line, up to 4094 bytes; use the full bridge for larger requests. */
int main(void) {
    char request[4096];
    if (ge4g_abi_version() != 1) {
        fputs("Expected GE4G ABI 1\n", stderr);
        return 1;
    }
    while (fgets(request, sizeof request, stdin)) {
        if (!strchr(request, '\n') && !feof(stdin)) {
            fputs("Request exceeds one input line\n", stderr);
            return 1;
        }
        char *response = ge4g_request_json(request);
        if (!response) {
            fputs("GE4G returned no response\n", stderr);
            return 1;
        }
        puts(response);
        ge4g_free_string(response);
        if (fflush(stdout) == EOF) return 1;
    }
    return ferror(stdin) ? 1 : 0;
}
