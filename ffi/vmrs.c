#include "vmrs.h"

/**
 * Main function to initialize the library.
 *
 * Needs to run on startup and is NOT thread-safe.
 */
void vmrs_initialize() {
    av_register_all();
}

/**
 * Frees the fields of a single metadata struct.
 */
void vmrs_metadata_free(struct vmrs_metadata* metadata) {
    if (!metadata)
        return;

    if (metadata->video_codec) {
        free(metadata->video_codec);
        metadata->video_codec = NULL;
    }

    if (metadata->audio_codec) {
        free(metadata->audio_codec);
        metadata->audio_codec = NULL;
    }

    if (metadata->format) {
        free(metadata->format);
        metadata->format = NULL;
    }
}

/**
 * This is the opaque structure we pass to the reading callback for our custom
 * context, in order to fake we're reading from a file (though we're actually
 * reading from a buffer).
 */
struct buffer_data {
    const uint8_t* ptr;
    size_t size;
};

static int read_packet(void *opaque, uint8_t *buf, int buf_size) {
    struct buffer_data* data = (struct buffer_data *)opaque;
    if (!opaque || buf_size < 0)
        return 0; // This shouldn't ever happen, but anyway.

    size_t real_buf_size = FFMIN((size_t)buf_size, data->size);

    /**
     * So now the magic happens, we copy as much data as we can from our
     * buffer_data struct (that has the provided data), and copy it to the
     * library buffer.
     *
     * We leave the pointer and the size with the next position to seek.
     */
    memcpy(buf, data->ptr, real_buf_size);

    data->ptr  += real_buf_size;
    data->size -= real_buf_size;

    // NB: real_buf_size can never overflow because of the FFMIN.
    return (int) real_buf_size;
}

const size_t VMRS_INITIAL_BUFFER_SIZE = 4096;

/**
 * Read metadata from either a buffer and a size, or a filename
 *
 * If buffer is non-NULL, then size must be > 0, and filename must be null.
 *
 * Note that we return VMRS_RESULT_OK on success, bigger errors for errors we've
 * handled, and negative values for libav errors.
 *
 * This might get a bit tricky though.
 */
int vmrs_read_info(const uint8_t* buffer,
                   uint32_t size,
                   const char* filename,
                   struct vmrs_metadata* out) {
    if (!out)
        return VMRS_ERROR_INPUT_FAILURE;

    // Only one of both, sorry.
    if (buffer && filename)
        return VMRS_ERROR_INPUT_FAILURE;

    // Please give me at least one byte.
    if (buffer && !size)
        return VMRS_ERROR_INPUT_FAILURE;

    AVFormatContext* format_ctx = NULL;
    AVIOContext* io_ctx = NULL;

    format_ctx = avformat_alloc_context();
    if (!format_ctx)
        return VMRS_ERROR_ALLOC;

    // If we're provided with a buffer, we want to create a custom audio context
    // that fakes the "read_packet" operation, see the `read_packet` function
    // above.
    struct buffer_data buffer_data;
    if (buffer) {
        buffer_data.size = size;
        buffer_data.ptr = buffer;

        // Create a buffer with av_malloc for libav to be happy.
        unsigned char* avio_ctx_buffer = av_malloc(VMRS_INITIAL_BUFFER_SIZE);
        if (!avio_ctx_buffer) {
            avformat_close_input(&format_ctx);
            return VMRS_ERROR_ALLOC;
        }
        io_ctx = avio_alloc_context(avio_ctx_buffer, VMRS_INITIAL_BUFFER_SIZE,
                                    /* writeable = */ 0,
                                    /* opaque = */ &buffer_data,
                                    &read_packet, NULL, NULL);
        if (!io_ctx) {
            avformat_close_input(&format_ctx);
            return VMRS_ERROR_ALLOC;
        }

        // Set the current format context's I/O context before
        // avformat_open_input.
        //
        // Note that we have to close it manually then, see
        // https://ffmpeg.org/doxygen/trunk/structAVFormatContext.html
        format_ctx->pb = io_ctx;
    }

    // We'll use this to ease the exit, since otherwise the code gets a bit
    // tricky.
    int ret = VMRS_RESULT_OK;

    // We've got to declare these here in order to be allowed to use goto...
    // Sorry about that.
    AVCodec* audio_decoder = NULL;
    AVCodec* video_decoder = NULL;
    AVCodecContext* video_codec_context = NULL;
    int video_stream_index;
    int audio_stream_index;

    ret = avformat_open_input(&format_ctx, filename, NULL, NULL);
    if (ret < 0) {
        // NB: avformat_open_input already frees the context on failure, though
        // doesn't free io_context if it's a custom one.
        format_ctx = NULL;
        // NB: We return the `ret` value, which is the real libav format.
        goto errorexit;
    }

    assert(format_ctx);
    if (!format_ctx->iformat || !format_ctx->iformat->name) {
        ret = VMRS_FORMAT_NOT_AVAILABLE;
        goto errorexit;
    }

    ret = avformat_find_stream_info(format_ctx, NULL);
    if (ret < 0)
        goto errorexit;

    video_stream_index = av_find_best_stream(format_ctx, AVMEDIA_TYPE_VIDEO,
                                             /* wanted_stream = */ -1,
                                             /* related_stream = */ -1,
                                             /* decoder_ret = */ &video_decoder,
                                             /* flags = */ 0);
    if (video_stream_index < 0) {
        ret = video_stream_index;
        goto errorexit;
    }

    // We're basically done here. We'll check in case we have an audio decoder,
    // but who cares.
    audio_stream_index = av_find_best_stream(format_ctx, AVMEDIA_TYPE_AUDIO,
                                             /* wanted_stream = */ -1,
                                             /* related_stream = */ -1,
                                             /* decoder_ret = */ &audio_decoder,
                                             /* flags = */ 0);

    out->audio_codec = NULL;
    if (audio_stream_index >= 0) { // Ignore errors.
        assert(audio_decoder);
        if (audio_decoder->name)
            out->audio_codec = strdup(audio_decoder->name);
        else if (audio_decoder->long_name)
            out->audio_codec = strdup(audio_decoder->long_name);
    }


    // av_find_best_stream already returns AVERROR_DECODER_NOT_FOUND in this
    // case, so we should always have a decoder.
    assert(video_decoder);

    video_codec_context = format_ctx->streams[video_stream_index]->codec;
    assert(video_codec_context);

    if (video_decoder->name)
        out->video_codec = strdup(video_decoder->name);
    else if (video_decoder->long_name)
        out->video_codec = strdup(video_decoder->long_name);
    else
        out->video_codec = NULL;

    out->width = video_codec_context->width;
    out->height = video_codec_context->height;
    out->delay = video_codec_context->delay;
    out->duration = format_ctx->duration;

    // Try again, just in case
    if (out->duration < 0)
        out->duration = format_ctx->streams[video_stream_index]->duration;

    out->format = strdup(format_ctx->iformat->name);

    ret = VMRS_RESULT_OK;

errorexit:
    if (format_ctx) {
        avformat_close_input(&format_ctx);
        io_ctx = NULL; // Already freed, see below.
    }

    // Quoting ffmpeg docs: AVIOContext.buffer holds the buffer currently in
    // use, which must be later freed with av_free().
    //
    // It seems though, that the underlying aio_close expects a valid buffer, so
    // doing it causes a segfault.
    // if (io_ctx->buffer) {
    //     av_free(io_ctx->buffer);
    //     io_ctx->buffer = NULL;
    // }
    // avio_close(io_ctx);

    return ret;
}

int vmrs_read_info_from_buffer(const uint8_t* buffer,
                               uint32_t size,
                               struct vmrs_metadata* out) {
    return vmrs_read_info(buffer, size, NULL, out);
}

int vmrs_read_info_from_file(const char* filename,
                             struct vmrs_metadata* out) {
    return vmrs_read_info(NULL, 0, filename, out);
}
