#ifndef INTERNALS_H_
#define INTERNALS_H_

#include <stdint.h>
#include <stdlib.h>
#include <stddef.h>

#define FFMIN(a, b) ((a) > (b) ? (b) : (a))

/**
 * This is the opaque structure we pass to the reading callback for our custom
 * context, in order to fake we're reading from a file (though we're actually
 * reading from a buffer).
 */
struct buffer_data {
    const uint8_t* ptr;
    size_t size;
};

enum sym_name {
    AV_ALLOC_CONTEXT = 0,
    AV_MALLOC,
    AV_CLOSE_INPUT,
    AV_IO_ALLOC_CONTEXT,
    AV_OPEN_INPUT,
    AV_FIND_STREAM_INFO,
    AV_FIND_BEST_STREAM,
    AV_REGISTER_ALL,
    AV_STR_ERROR,
    SYMBOLS_NUMBER
};

 enum AVMediaType {
    AVMEDIA_TYPE_UNKNOWN = -1,  ///< Usually treated as AVMEDIA_TYPE_DATA
    AVMEDIA_TYPE_VIDEO,
    AVMEDIA_TYPE_AUDIO,
    AVMEDIA_TYPE_DATA,          ///< Opaque data information usually continuous
    AVMEDIA_TYPE_SUBTITLE,
    AVMEDIA_TYPE_ATTACHMENT,    ///< Opaque data information usually sparse
    AVMEDIA_TYPE_NB
};

typedef struct AVRational {
    int num;
    int den;
} AVRational;

typedef struct AVCodec {
    char *name;
    char *long_name;
} AVCodec;

typedef struct AVCodecContext {
    void *av_class;
    int log_level_offset;
    int codec_type;
    const struct AVCodec *codec;
    char codec_name[32];
    int codec_id;
    unsigned int codec_tag;
    unsigned int stream_codec_tag;
    void *priv_data;
    void *internal;
    void *opaque;
    int64_t bit_rate;
    int bit_rate_tolerance;
    int global_quality;
    int compression_level;
    int flags;
    int flags2;
    uint8_t *extradata;
    int extradata_size;
    AVRational time_base;
    int ticks_per_frame;
    int delay;
    int width;
    int height;
} AVCodecContext;

typedef struct AVStream {
    int index;
    int id;
    AVCodecContext *codec;
    void *priv_data;
    int64_t start_time;
    int64_t duration;
} AVStream;

typedef struct AVInputFormat {
    const char *name;
} AVInputFormat;

typedef struct AVFormatContext {
    void *av_class;
    AVInputFormat *iformat;
    void *oformat;
    void *priv_data;
    void *pb;
    int ctx_flags;
    unsigned int nb_streams;
    AVStream **streams;
    char filename[1024];
    int64_t start_time;
    int64_t duration;
} AVFormatContext;

#define av_alloc_context() ((AVFormatContext*(*)())symbols[AV_ALLOC_CONTEXT])()
#define av_malloc(x) ((unsigned char*(*)(int))symbols[AV_MALLOC])(x)
#define av_close_input(x) ((void(*)(AVFormatContext**))symbols[AV_CLOSE_INPUT])(x)
#define avio_alloc_context(a,b,c,d,e,f,g) ((void*(*)(unsigned char*, int, int, void*,void*,void*,void*))symbols[AV_IO_ALLOC_CONTEXT])(a,b,c,d,e,f,g)
#define av_open_input(a,b,c,d) ((int(*)(AVFormatContext**,const char*, void*, void*))symbols[AV_OPEN_INPUT])(a,b,c,d)
#define av_find_stream_info(x,y) ((int(*)(AVFormatContext*, void*))symbols[AV_FIND_STREAM_INFO])(x,y)
#define av_find_best_stream(a,b,c,d,e,f) ((int(*)(AVFormatContext*, int, int, int, AVCodec**,int))symbols[AV_FIND_BEST_STREAM])(a,b,c,d,e,f)
#define av_str_error(x,y,z) ((int(*)(int, char*, size_t))symbols[AV_STR_ERROR])(x,y,z)
#define av_register_all() ((void(*)())symbols[AV_REGISTER_ALL])()

#endif
