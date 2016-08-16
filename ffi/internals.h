#ifndef INTERNALS_H_
#define INTERNALS_H_

/**
 * This is the opaque structure we pass to the reading callback for our custom
 * context, in order to fake we're reading from a file (though we're actually
 * reading from a buffer).
 */
struct buffer_data {
    const uint8_t* ptr;
    size_t size;
};

typedef enum _sym_name {
    AV_ALLOC_CONTEXT = 0,
    AV_MALLOC,
    AV_CLOSE_INPUT,
    AV_IO_ALLOC_CONTEXT,
    AV_OPEN_INPUT,
    AV_FIND_STREAM_INFO,
    AV_FIND_BEST_STREAM
} sym_name;

#endif INTERNALS_H_