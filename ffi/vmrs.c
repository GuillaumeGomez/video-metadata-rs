#include <dlfcn.h>
#include "internals.h"
#include "vmrs.h"

int free_dlsyms(void *avformat_link, void *avcodec_link, void *avutil_link, int ret) {
    if (avformat_link) {
        dlclose(avformat_link);
    }
    if (avcodec_link) {
        dlclose(avcodec_link);
    }
    if (avutil_link) {
        dlclose(avutil_link);
    }
    return ret;
}

int get_symbol(void *linker, const char *name, void **tab) {
    *tab = dlsym(linker, name);
    return *tab != NULL;
}

int get_symbols(void *avformat_link, void *avcodec_link, void *avutil_link, void **symbols) {
    return get_symbol(avformat_link, "avformat_alloc_context", &symbols[AV_ALLOC_CONTEXT]) &&
           get_symbol(avutil_link, "av_malloc", &symbols[AV_MALLOC]) &&
           get_symbol(avformat_link, "avformat_close_input", &symbols[AV_CLOSE_INPUT]) &&
           get_symbol(avformat_link, "avio_alloc_context", &symbols[AV_IO_ALLOC_CONTEXT]) &&
           get_symbol(avformat_link, "avformat_open_input", &symbols[AV_OPEN_INPUT]) &&
           get_symbol(avformat_link, "avformat_find_stream_info", &symbols[AV_FIND_STREAM_INFO]) &&
           get_symbol(avformat_link, "av_find_best_stream", &symbols[AV_FIND_BEST_STREAM]);
}

int av_strerror(int errnum, char *errbuf, size_t errbuf_size) {
    void *avutil_link = dlopen("libavutil.so", RTLD_LAZY);
    if (!avutil_link) {
        return 0;
    }
    void *symbol = dlsym(avutil_link, "av_strerror");
    if (!symbol) {
        return 0;
    }
    int ret = ((int(*)(int, char*, size_t))symbol)(errnum, errbuf, errbuf_size);
    dlclose(avutil_link);
    return ret;
}

/**
 * Main function to initialize the library.
 *
 * Needs to run on startup and is NOT thread-safe.
 */
void vmrs_initialize() {
    void *avutil_link = dlopen("libavutil.so", RTLD_LAZY);
    if (!avutil_link) {
        return;
    }
    void *symbol = dlsym(avutil_link, "av_register_all");
    if (!symbol) {
        return;
    }
    ((void(*)())symbol)();
    dlclose(avutil_link);
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

    // First we get libraries handler.
    void *avformat_link, *avcodec_link, *avutil_link;
    avformat_link = avcodec_link = avutil_link = NULL;
    if (!(avformat_link = dlopen("libavformat.so", RTLD_LAZY)) ||
        !(avcodec_link = dlopen("libavcodec.so", RTLD_LAZY)) ||
        !(avutil_link = dlopen("libavutil.so", RTLD_LAZY))) {
        return free_dlsyms(avformat_link, avcodec_link, NULL, VMRS_LIB_NOT_FOUND);
    }

    // Then we get functions symbol.
    void *symbols[7] = {NULL, NULL, NULL, NULL, NULL, NULL, NULL};
    if (!get_symbols(avformat_link, avcodec_link, avutil_link, symbols)) {
        return free_dlsyms(avformat_link, avcodec_link, avutil_link, VMRS_FUNC_NOT_FOUND);
    }

    AVFormatContext* format_ctx = NULL;
    void* io_ctx = NULL;

    format_ctx = ((AVFormatContext*(*)())symbols[AV_ALLOC_CONTEXT])();
    if (!format_ctx) {
        return VMRS_ERROR_ALLOC;
    }

    // If we're provided with a buffer, we want to create a custom audio context
    // that fakes the "read_packet" operation, see the `read_packet` function
    // above.
    if (buffer) {
        struct buffer_data buffer_data;

        buffer_data.size = size;
        buffer_data.ptr = buffer;

        // Create a buffer with av_malloc for libav to be happy.
        unsigned char* avio_ctx_buffer = ((unsigned char*(*)(int))symbols[AV_MALLOC])(VMRS_INITIAL_BUFFER_SIZE);
        if (!avio_ctx_buffer) {
            ((void(*)(AVFormatContext**))symbols[AV_CLOSE_INPUT])(&format_ctx);
            return VMRS_ERROR_ALLOC;
        }
        io_ctx = ((void*(*)(unsigned char*, int, int, void*, void*, void*, void*))symbols[AV_IO_ALLOC_CONTEXT])(
            avio_ctx_buffer,
            VMRS_INITIAL_BUFFER_SIZE,
            /* writeable = */ 0,
            /* opaque = */ &buffer_data,
            &read_packet,
            NULL,
            NULL);
        if (!io_ctx) {
            ((void(*)(AVFormatContext**))symbols[AV_CLOSE_INPUT])(&format_ctx);
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

    ret = ((int(*)(AVFormatContext**,const char*, void*, void*))symbols[AV_OPEN_INPUT])(
        &format_ctx, filename, NULL, NULL);
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

    ret = ((int(*)(AVFormatContext*, void*))symbols[AV_FIND_STREAM_INFO])(format_ctx, NULL);
    if (ret < 0)
        goto errorexit;

    video_stream_index = ((int(*)(AVFormatContext*, int, int, int, AVCodec**, int))symbols[AV_FIND_BEST_STREAM])(
        format_ctx,
        AVMEDIA_TYPE_VIDEO,
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
    audio_stream_index = ((int(*)(AVFormatContext*, int, int, int, AVCodec**, int))symbols[AV_FIND_BEST_STREAM])(
        format_ctx,
        AVMEDIA_TYPE_AUDIO,
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
        ((void(*)(AVFormatContext**))symbols[AV_CLOSE_INPUT])(&format_ctx);
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

    return free_dlsyms(avformat_link, avcodec_link, avutil_link, ret);
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
