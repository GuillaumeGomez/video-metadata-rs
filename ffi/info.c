#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libavutil/opt.h>

#include <stdlib.h>
#include <string.h>

static int initialized = 0;

struct Metadata {
    int64_t duration;
    unsigned int width;
    unsigned int height;
    char *video_codec;
    unsigned int video_codec_len;
    char *audio_codec;
    unsigned int audio_codec_len;
    char *format;
    unsigned int format_len;
};

struct Ret {
    struct Metadata *m;
    char *error;
    unsigned int error_len;
};

static AVCodecContext *iterate_over_streams(AVFormatContext *ic, int i) {
    AVCodecContext *avctx = avcodec_alloc_context3(NULL);
    if (!avctx) {
        return NULL;
    }
    if (avcodec_parameters_to_context(avctx, ic->streams[i]->codecpar) < 0) {
        avcodec_free_context(&avctx);
        return NULL;
    }
    return avctx;
}

static struct Ret create_ret(struct Metadata *m, char *err) {
    struct Ret r;

    r.m = m;
    r.error = err;
    if (err) {
        r.error_len = strlen(err);
    }
    return r;
}

static void free_metadata(struct Metadata **m) {
    if (!m || !m[0]) {
        return;
    }
    free(m[0]->video_codec);
    free(m[0]->audio_codec);
    free(m[0]->format);
    free(m[0]);
    m[0] = NULL;
}

void free_ret(struct Ret *r) {
    if (!r) {
        return;
    }
    free_metadata(&r->m);
    free(r->error);
}

struct Ret get_information(const char *filename) {
    int x;
    if (!initialized) {
        av_register_all();
        initialized = 1;
    }
    struct Metadata *m = calloc(1, sizeof(struct Metadata));
    if (!m) {
        return create_ret(NULL, strdup("metadata allocation failed"));
    }

    AVFormatContext *ic = NULL;
    int err = avformat_open_input(&ic, filename, NULL, NULL);
    if (err < 0) {
        char errbuf[129] = {0};
        const char *errbuf_ptr = errbuf;

        if (av_strerror(err, errbuf, sizeof(errbuf) - 1) < 0) {
            errbuf_ptr = strerror(AVUNERROR(err));
        }
        free(m);
        return create_ret(NULL, strdup(errbuf_ptr));
    }

    if (ic->iformat && ic->iformat->name) {
        m->format = strdup(ic->iformat->name);
    } else {
        free_metadata(&m);
        avformat_free_context(ic);
        return create_ret(NULL, strdup("Unable to get format"));
    }
    m->format_len = strlen(m->format);
    m->duration = ic->duration;
    for (x = 0; x < ic->nb_streams; x++) {
        AVCodecContext *avctx = iterate_over_streams(ic, x);
        if (avctx) {
            if (avctx->codec_type == AVMEDIA_TYPE_VIDEO) {
                if (avctx->width) {
                    m->width = avctx->width;
                    m->height = avctx->height;
                }
                m->video_codec = strdup(avcodec_get_name(avctx->codec_id));
                m->video_codec_len = strlen(m->video_codec);
            } else if (avctx->codec_type == AVMEDIA_TYPE_AUDIO) {
                m->audio_codec = strdup(avcodec_get_name(avctx->codec_id));
                m->audio_codec_len = strlen(m->audio_codec);
            }
            avcodec_free_context(&avctx);
        }
    }
    avformat_close_input(&ic);
    if (!m->video_codec) {
        free_metadata(&m);
        return create_ret(NULL, strdup("Unable to get video codec"));
    }
    return create_ret(m, NULL);
}

int64_t get_time_base(void) {
    return AV_TIME_BASE;
}
/*
// to test just this file: gcc info.c -lavcodec -lavformat -lavutil
void print_ret(struct Ret *r) {
    if (!r->m) {
        if (!r->error) {
            printf("unknown error\n");
        } else {
            printf("error: %s\n", r->error);
        }
    } else {
        printf("duration: %ld\n", r->m->duration);
        printf("size    : %dx%d\n", r->m->width, r->m->height);
        printf("video   : %s\n", r->m->video_codec);
        if (r->m->audio_codec) {
            printf("audio   : %s\n", r->m->audio_codec);
        }
        printf("format  : %s\n", r->m->format);
    }
}

void print_info(const char *type, const char *filename) {
    printf("====== %s ======\n", type);
    struct Ret r = get_information(filename);
    print_ret(&r);
    free_ret(&r);
}

int main() {
    print_info("WEBM", "../assets/big-buck-bunny_trailer.webm");
    print_info("MP4", "../assets/small.mp4");
    print_info("OGG", "../assets/small.ogg");
    return 0;
}*/
