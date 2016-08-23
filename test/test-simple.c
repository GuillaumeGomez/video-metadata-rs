#include <dlfcn.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "ffi/vmrs.h"

// This is a simple test program to double-check we don't have leaks.
//
// Compile from the root of the crate with:
//
// gcc test/test-simple.c -I. -L target/debug/build/video-metadata-*/out/ -lvmrs -lavcodec -lavutil -lavformat
int main() {
    struct vmrs_metadata metadata;

    char *ffmpeg_lib_dir = getenv("FFMPEG_LIB_DIR");
    void *avformat_link = NULL;
    void *avutil_link = NULL;
    void *syms[9] = {NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL};

    if (ffmpeg_lib_dir) {
        char avformat_lib_path[256] = {0};
        char avutil_lib_path[256] = {0};
        int length = strlen(ffmpeg_lib_dir);

        if (length > 1 && ffmpeg_lib_dir[length - 1] == '/') {
            snprintf(avformat_lib_path, 255, "%s%s", ffmpeg_lib_dir, "libavformat.so");
            snprintf(avutil_lib_path, 255, "%s%s", ffmpeg_lib_dir, "libavutil.so");
        } else {
            snprintf(avformat_lib_path, 255, "%s/%s", ffmpeg_lib_dir, "libavformat.so");
            snprintf(avutil_lib_path, 255, "%s/%s", ffmpeg_lib_dir, "libavutil.so");
        }
        printf("'%s' / '%s'\n", avformat_lib_path, avutil_lib_path);
        avformat_link = dlopen(avformat_lib_path, RTLD_LAZY);
        avutil_link = dlopen(avutil_lib_path, RTLD_LAZY);
    } else {
        avformat_link = dlopen("libavformat.so", RTLD_LAZY);
        avutil_link = dlopen("libavutil.so", RTLD_LAZY);

        if (!avformat_link || !avutil_link) {
            avformat_link = dlopen("/usr/local/lib/libavformat.so", RTLD_LAZY);
            avutil_link = dlopen("/usr/local/lib/libavutil.so", RTLD_LAZY);
        }
    }

    if (!avformat_link || !avutil_link) {
        fprintf(stderr, "%s\n", "Libraries not found");
        return 1;
    }

    syms[0] = dlsym(avformat_link, "avformat_alloc_context");
    syms[1] = dlsym(avutil_link, "av_malloc");
    syms[2] = dlsym(avformat_link, "avformat_close_input");
    syms[3] = dlsym(avformat_link, "avio_alloc_context");
    syms[4] = dlsym(avformat_link, "avformat_open_input");
    syms[5] = dlsym(avformat_link, "avformat_find_stream_info");
    syms[6] = dlsym(avformat_link, "av_find_best_stream");
    syms[7] = dlsym(avformat_link, "av_register_all");
    syms[8] = dlsym(avutil_link, "av_strerror");

    vmrs_initialize(syms);

    int ret = vmrs_read_info_from_file("assets/small.ogg", &metadata, syms);
    if (ret != 0) {
        dlclose(avformat_link);
        dlclose(avutil_link);
        fprintf(stderr, "Got error: %d\n", ret);
        return 1;
    }

    printf("format: \"%s\" vcodec: \"%s\" acodec: \"%s\"\n", metadata.format,
                                                             metadata.video_codec,
                                                             metadata.audio_codec);

    vmrs_metadata_free(&metadata);

    ret = vmrs_read_info_from_file("assets/non-existent.txt", &metadata, syms);
    dlclose(avformat_link);
    dlclose(avutil_link);
    if (ret != 0) {
        printf("Got error (as expected): %d\n", ret);
        return 0;
    }

    printf("Definitely shouldn't be here\n");
    return 1;
}
