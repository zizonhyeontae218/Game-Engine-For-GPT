#define _POSIX_C_SOURCE 200809L
#include "ge4g_p4_v1.h"
#include <stdio.h>
#include <stdlib.h>
#include <sys/types.h>

/* Standalone native QA executable, no APK, installation or application data. */
int main(void) {
    char *line=NULL;size_t capacity=0;ssize_t n;
    if(ge4g_p4_v1_abi_version()!=1)return 2;
    while((n=getline(&line,&capacity,stdin))!=-1) {
        if(n>0 && line[n-1]=='\n')n--;
        char *response=ge4g_p4_v1_request((const uint8_t *)line,(size_t)n);
        if(!response)return 3;
        puts(response);ge4g_p4_v1_string_free(response);fflush(stdout);
    }
    free(line);return ferror(stdin)?4:0;
}
