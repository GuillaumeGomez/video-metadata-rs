#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libavutil/opt.h>

#include <stdlib.h>
#include <string.h>

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

void free_metadata(struct Metadata **m) {
    if (!m || !m[0]) {
        return;
    }
    free(m[0]->video_codec);
    free(m[0]->audio_codec);
    free(m[0]->format);
    free(m[0]);
    m[0] = NULL;
}

int check_stream_specifier(AVFormatContext *s, AVStream *st, const char *spec)
{
    int ret = avformat_match_stream_specifier(s, st, spec);
    if (ret < 0)
        av_log(s, AV_LOG_ERROR, "Invalid stream specifier: %s.\n", spec);
    return ret;
}

AVDictionary *filter_codec_opts(AVDictionary *opts, enum AVCodecID codec_id,
                                AVFormatContext *s, AVStream *st, AVCodec *codec)
{
    AVDictionary    *ret = NULL;
    AVDictionaryEntry *t = NULL;
    int            flags = s->oformat ? AV_OPT_FLAG_ENCODING_PARAM
                                      : AV_OPT_FLAG_DECODING_PARAM;
    char          prefix = 0;
    const AVClass    *cc = avcodec_get_class();

    if (!codec)
        codec            = s->oformat ? avcodec_find_encoder(codec_id)
                                      : avcodec_find_decoder(codec_id);

    switch (st->codec->codec_type) {
    case AVMEDIA_TYPE_VIDEO:
        prefix  = 'v';
        flags  |= AV_OPT_FLAG_VIDEO_PARAM;
        break;
    case AVMEDIA_TYPE_AUDIO:
        prefix  = 'a';
        flags  |= AV_OPT_FLAG_AUDIO_PARAM;
        break;
    case AVMEDIA_TYPE_SUBTITLE:
        prefix  = 's';
        flags  |= AV_OPT_FLAG_SUBTITLE_PARAM;
        break;
    }

    while (t = av_dict_get(opts, "", t, AV_DICT_IGNORE_SUFFIX)) {
        char *p = strchr(t->key, ':');

        /* check stream specification in opt name */
        if (p)
            switch (check_stream_specifier(s, st, p + 1)) {
            case  1: *p = 0; break;
            case  0:         continue;
            }

        if (av_opt_find(&cc, t->key, NULL, flags, AV_OPT_SEARCH_FAKE_OBJ) ||
            !codec ||
            (codec->priv_class &&
             av_opt_find(&codec->priv_class, t->key, NULL, flags,
                         AV_OPT_SEARCH_FAKE_OBJ)))
            av_dict_set(&ret, t->key, t->value, 0);
        else if (t->key[0] == prefix &&
                 av_opt_find(&cc, t->key + 1, NULL, flags,
                             AV_OPT_SEARCH_FAKE_OBJ))
            av_dict_set(&ret, t->key + 1, t->value, 0);

        if (p)
            *p = ':';
    }
    return ret;
}

AVDictionary **setup_find_stream_info_opts(AVFormatContext *s,
                                           AVDictionary *codec_opts)
{
    int i;
    AVDictionary **opts;

    if (!s->nb_streams)
        return NULL;
    opts = av_mallocz_array(s->nb_streams, sizeof(*opts));
    if (!opts) {
        av_log(NULL, AV_LOG_ERROR,
               "Could not alloc memory for stream options.\n");
        return NULL;
    }
    for (i = 0; i < s->nb_streams; i++)
        opts[i] = filter_codec_opts(codec_opts, s->streams[i]->codec->codec_id,
                                    s, s->streams[i], NULL);
    return opts;
}

AVCodecContext *iterate_over_streams(AVFormatContext *ic, int i) {
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

struct Metadata *get_information(const char *filename) {
    int x;
    struct Metadata *m = calloc(1, sizeof(struct Metadata));
    if (!m) {
        return NULL;
    }

    FILE *f = fopen("/tmp/p", "w");
    fprintf(f, "reading: '%s'!!!\n", filename);

    AVFormatContext *ic = NULL;
    AVInputFormat fmt = { 0 };
    AVDictionary *options = NULL;
    av_dict_set(&options, "scan_all_pmts", "1", AV_DICT_DONT_OVERWRITE);
    if (avformat_open_input(&ic, filename, &fmt, &options) < 0) {
        free(m);
        av_dict_free(&options);
        return NULL;
    }
    fprintf(f, "streams: %d\nfile: %s\nduration: %d\nbitrate: %d\n",
            ic->nb_streams, ic->filename, ic->duration, ic->bit_rate);
    fflush(f);

    AVDictionary **opts = setup_find_stream_info_opts(ic, NULL);
    int err = avformat_find_stream_info(ic, opts);
    for (x = 0; x < ic->nb_streams; x++)
        av_dict_free(&opts[x]);
    av_freep(&opts);
    if (err < 0) {
        free(m);
        av_dict_free(&options);
        return NULL;
    }

    if (ic->iformat && ic->iformat->name) {
        m->format = strdup(ic->iformat->name);
    } else if (ic->oformat && ic->oformat->name) {
        m->format = strdup(ic->oformat->name);
    }
    if (m->format) {
        m->format_len = strlen(m->format);
    }
    m->duration = ic->duration;
    for (x = 0; x < ic->nb_streams; x++) {
        AVCodecContext *avctx = iterate_over_streams(ic, x);
        if (avctx) {
            if (avctx->codec_type == AVMEDIA_TYPE_VIDEO) {
                if (avctx->width) {
                    m->width = avctx->width;
                    m->height = avctx->height;
                }
                fprintf(f, "video!!!\n");
                m->video_codec = strdup(avcodec_get_name(avctx->codec_id));
                fprintf(f, "video_codec: %s\n", m->video_codec);
            } else if (avctx->codec_type == AVMEDIA_TYPE_AUDIO) {
                fprintf(f, "audio!!!\n");
                m->audio_codec = strdup(avcodec_get_name(avctx->codec_id));
            }
            avcodec_free_context(&avctx);
        }
    }
    fclose(f);
    av_freep(&ic->streams);
    //avformat_close_input(&ic);
    av_dict_free(&options);
    if (!m->video_codec) {
        free_metadata(&m);
        return NULL;
    }
    return m;
}

int64_t get_time_base(void) {
    return AV_TIME_BASE;
}
