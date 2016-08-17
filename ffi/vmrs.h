#ifndef VMRS_H_
#define VMRS_H_

/*#include <libavformat/avformat.h>
#include <libavformat/avio.h>
#include <libavcodec/avcodec.h>
#include <libavutil/file.h>
#include <libavutil/opt.h>*/

#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <assert.h>

/**
 * Main video metadata structure for communicating video metadata information
 * between Rust and C.
 *
 * Each field is self-descriptive, and the pointer fields are owned and need to
 * be freed apropriately with vmrs_metadata_free.
 */
struct vmrs_metadata {
    int64_t duration;

    uint32_t width;
    uint32_t height;
    uint32_t delay;

    char* video_codec;
    char* audio_codec;
    char* format;
};

/**
 * The main result enum.
 *
 * Note that this is an enum for convenience, but in practice we return an
 * `int` to take into account libav errors.
 */
typedef enum vmrs_result {
    VMRS_RESULT_OK = 0,         ///< Everything went all-right
    VMRS_ERROR_INPUT_FAILURE,   ///< Some input provided to the library wasn't the expected.
    VMRS_ERROR_ALLOC,           ///< Wasn't able to allocate something.
    VMRS_FORMAT_NOT_AVAILABLE,  ///< Wasn't able to get the input format.
    VMRS_LIB_NOT_FOUND,         ///< In case dlopen cannot find one of the needed libraries.
    VMRS_FUNC_NOT_FOUND         ///< If a function isn't found in a library.
} vmrs_result_t;

void vmrs_initialize(void **symbols);

int vmrs_read_info_from_buffer(const uint8_t* buffer,
                               uint32_t size,
                               struct vmrs_metadata* out,
                               void **symbols);

int vmrs_read_info_from_file(const char* filename,
                             struct vmrs_metadata* out,
                             void **symbols);

void vmrs_metadata_free(struct vmrs_metadata* metadata);

void *get_lib_handler(const char *name);

#endif
