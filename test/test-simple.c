#include "ffi/vmrs.h"

// This is a simple test program to double-check we don't have leaks.
//
// Compile from the root of the crate with:
//
// gcc test/test-simple.c -I. -L target/debug/build/video-metadata-*/out/ -lvmrs -lavcodec -lavutil -lavformat
int main() {
    struct vmrs_metadata metadata;

    vmrs_initialize();

    int ret = vmrs_read_info_from_file("assets/small.ogg", &metadata);
    if (ret != 0) {
        printf("Got error: %d\n", ret);
        return 1;
    }

    printf("format: \"%s\" vcodec: \"%s\" acodec: \"%s\"\n", metadata.format,
                                                             metadata.video_codec,
                                                             metadata.audio_codec);

    vmrs_metadata_free(&metadata);

    ret = vmrs_read_info_from_file("assets/non-existent.txt", &metadata);
    if (ret != 0) {
        printf("Got error (as expected): %d\n", ret);
        return 0;
    }

    printf("Definitely shouldn't be here\n");
    return 1;
}
