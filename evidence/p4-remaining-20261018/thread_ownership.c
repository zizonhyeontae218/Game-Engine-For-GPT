#include "ge4g_p4_v1.h"
#include <pthread.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>

static void *worker(void *ignored) {
    (void)ignored;
    const char *request="{\"version\":1,\"command\":{\"op\":\"create\",\"config\":{\"version\":1,\"width\":160,\"height\":96,\"tick_hz\":60,\"attack_control\":\"attack\",\"fighters\":[{\"id\":\"blue\",\"hp\":12,\"damage\":4,\"position\":[40,72],\"color\":[48,144,255,255]},{\"id\":\"red\",\"hp\":11,\"damage\":3,\"position\":[120,72],\"color\":[255,80,72,255]}]},\"seed\":42,\"mode\":\"linked\"}}";
    for (int i=0;i<16;i++) {
        char *r=ge4g_p4_v1_request((const uint8_t *)request,strlen(request));
        assert(r && strstr(r,"\"ok\":true"));ge4g_p4_v1_string_free(r);
    }
    return NULL; /* all 16 sessions must be reclaimed on pthread exit */
}
int main(void) {
    pthread_t threads[8];
    for(int i=0;i<8;i++)assert(pthread_create(&threads[i],NULL,worker,NULL)==0);
    for(int i=0;i<8;i++)assert(pthread_join(threads[i],NULL)==0);
    puts("PASS 128 owned responses freed; 128 sessions left for thread-exit cleanup");
}
