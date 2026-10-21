#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/hwcontext.h>
#include <libavutil/time.h>
#include <libswresample/swresample.h>
#include <libswscale/swscale.h>
#include <math.h>
#include <pulse/error.h>
#include <pulse/simple.h>
#include <stdlib.h>
#include <string.h>
// All objects are owned and destroyed on one worker thread. Callbacks never
// retain C pointers.
typedef int (*Control)(void *, int); // 0: cancellation; 1: audio volume 0..100
typedef void (*Video)(void *, const unsigned char *, int, int, int, int64_t);
typedef struct {
  void *user;
  Control control;
  int64_t deadline;
} Interrupt;
static int interrupt(void *opaque) {
  Interrupt *i = opaque;
  return i->control(i->user, 0) || av_gettime_relative() > i->deadline;
}
static const enum AVPixelFormat hw_format = AV_PIX_FMT_VAAPI;
static enum AVPixelFormat choose_format(AVCodecContext *ctx,
                                        const enum AVPixelFormat *formats) {
  (void)ctx;
  for (const enum AVPixelFormat *p = formats; *p != AV_PIX_FMT_NONE; p++)
    if (*p == hw_format)
      return *p;
  return formats[0];
}
static AVCodecContext *open_decoder(AVStream *stream, int hardware) {
  const AVCodec *codec = avcodec_find_decoder(stream->codecpar->codec_id);
  if (!codec)
    return NULL;
  AVCodecContext *ctx = avcodec_alloc_context3(codec);
  if (!ctx)
    return NULL;
  if (avcodec_parameters_to_context(ctx, stream->codecpar) < 0) {
    avcodec_free_context(&ctx);
    return NULL;
  }
  ctx->thread_count = 2;
  ctx->max_pixels = 8192LL * 8192;
  if (hardware) {
    int supported = 0;
    for (int n = 0;; n++) {
      const AVCodecHWConfig *c = avcodec_get_hw_config(codec, n);
      if (!c)
        break;
      if (c->device_type == AV_HWDEVICE_TYPE_VAAPI &&
          (c->methods & AV_CODEC_HW_CONFIG_METHOD_HW_DEVICE_CTX)) {
        supported = 1;
        break;
      }
    }
    if (supported &&
        av_hwdevice_ctx_create(&ctx->hw_device_ctx, AV_HWDEVICE_TYPE_VAAPI,
                               NULL, NULL, 0) >= 0)
      ctx->get_format = choose_format;
  }
  if (avcodec_open2(ctx, codec, NULL) < 0) {
    avcodec_free_context(&ctx);
    return NULL;
  }
  return ctx;
}
// Returns FFmpeg error codes, never raw URLs/errors that could expose
// credentials.
int cctv_decode(const char *url, int tcp, int timeout_ms, int hardware,
                int input_kind, int tls_verify, void *user, Control control,
                Video video) {
  int result = 0, vi = -1, ai = -1;
  AVFormatContext *fmt = avformat_alloc_context();
  AVCodecContext *vd = NULL, *ad = NULL;
  AVPacket *packet = av_packet_alloc();
  AVFrame *frame = av_frame_alloc(), *software = av_frame_alloc();
  struct SwsContext *scale = NULL;
  SwrContext *resample = NULL;
  pa_simple *audio = NULL;
  unsigned char *pixels = NULL;
  int pixels_size = 0;
  int64_t first_pts = AV_NOPTS_VALUE, clock_start = 0;
  Interrupt intr = {user, control,
                    av_gettime_relative() + (int64_t)timeout_ms * 1000};
  if (!fmt || !packet || !frame || !software) {
    result = AVERROR(ENOMEM);
    goto done;
  }
  fmt->interrupt_callback = (AVIOInterruptCB){interrupt, &intr};
  AVDictionary *options = NULL;
  av_dict_set(&options, "rtsp_transport", tcp ? "tcp" : "udp", 0);
  char timeout[32];
  snprintf(timeout, sizeof(timeout), "%lld", (long long)timeout_ms * 1000);
  av_dict_set(&options, "rw_timeout", timeout, 0);
  av_dict_set(&options, "timeout", timeout, 0);
  av_dict_set(&options, "tls_verify", tls_verify ? "1" : "0", 0);
  if (tls_verify)
    av_dict_set(&options, "ca_file", "/etc/ssl/certs/ca-certificates.crt", 0);
  av_dict_set(&options, "protocol_whitelist",
              "http,https,tcp,tls,rtsp,rtp,udp,crypto,data", 0);
  av_dict_set(&options, "allowed_extensions", "ALL", 0);
  // Stable Rust/C contract: 0 autodetect, 1 multipart JPEG, 2 raw MJPEG, 3
  // snapshot.
  const char *format_name = input_kind == 1   ? "mpjpeg"
                            : input_kind == 2 ? "mjpeg"
                            : input_kind == 3 ? "jpeg_pipe"
                                              : NULL;
  const AVInputFormat *input =
      format_name ? av_find_input_format(format_name) : NULL;
  if (format_name && !input) {
    av_dict_free(&options);
    result = AVERROR_DEMUXER_NOT_FOUND;
    goto done;
  }
  result = avformat_open_input(&fmt, url, input, &options);
  av_dict_free(&options);
  if (result < 0)
    goto done;
  result = avformat_find_stream_info(fmt, NULL);
  if (result < 0)
    goto done;
  vi = av_find_best_stream(fmt, AVMEDIA_TYPE_VIDEO, -1, -1, NULL, 0);
  if (vi < 0) {
    result = vi;
    goto done;
  }
  vd = open_decoder(fmt->streams[vi], hardware);
  if (!vd) {
    result = AVERROR_DECODER_NOT_FOUND;
    goto done;
  }
  ai = av_find_best_stream(fmt, AVMEDIA_TYPE_AUDIO, -1, -1, NULL, 0);
  if (ai >= 0)
    ad = open_decoder(fmt->streams[ai], 0);
  while (!control(user, 0)) {
    intr.deadline = av_gettime_relative() + (int64_t)timeout_ms * 1000;
    result = av_read_frame(fmt, packet);
    if (result < 0)
      break;
    AVCodecContext *decoder = packet->stream_index == vi
                                  ? vd
                                  : (packet->stream_index == ai ? ad : NULL);
    if (!decoder) {
      av_packet_unref(packet);
      continue;
    }
    result = avcodec_send_packet(decoder, packet);
    av_packet_unref(packet);
    if (result < 0)
      continue;
    while ((result = avcodec_receive_frame(decoder, frame)) >= 0) {
      if (decoder == vd) {
        AVFrame *src = frame;
        if (frame->format == AV_PIX_FMT_VAAPI) {
          av_frame_unref(software);
          if (av_hwframe_transfer_data(software, frame, 0) < 0) {
            result = AVERROR_EXTERNAL;
            goto done;
          }
          src = software;
        }
        control(user, frame->format == AV_PIX_FMT_VAAPI ? 2 : 3);
        int width = src->width, height = src->height;
        if (width <= 0 || height <= 0 || width > 8192 || height > 8192) {
          result = AVERROR_INVALIDDATA;
          goto done;
        }
        // Bound each panel to 1920px; consumers share a single latest-frame
        // buffer.
        int out_width = width > 1920 ? 1920 : width;
        int out_height = (int)((int64_t)height * out_width / width);
        int needed = out_width * out_height * 4;
        if (needed > pixels_size) {
          unsigned char *next = av_realloc(pixels, needed);
          if (!next) {
            result = AVERROR(ENOMEM);
            goto done;
          }
          pixels = next;
          pixels_size = needed;
        }
        scale = sws_getCachedContext(scale, width, height, src->format,
                                     out_width, out_height, AV_PIX_FMT_BGRA,
                                     SWS_FAST_BILINEAR, NULL, NULL, NULL);
        if (!scale) {
          result = AVERROR(ENOMEM);
          goto done;
        }
        unsigned char *dst[] = {pixels};
        int strides[] = {out_width * 4};
        sws_scale(scale, (const unsigned char *const *)src->data, src->linesize,
                  0, height, dst, strides);
        int64_t pts = frame->best_effort_timestamp;
        if (pts != AV_NOPTS_VALUE) {
          pts = av_rescale_q(pts, fmt->streams[vi]->time_base,
                             (AVRational){1, 1000000});
          if (first_pts == AV_NOPTS_VALUE) {
            first_pts = pts;
            clock_start = av_gettime_relative();
          }
          int64_t target = clock_start + pts - first_pts;
          // Pace buffered HTTP/HLS; do not accumulate latency when live
          // timestamps jump.
          if (target - av_gettime_relative() > 2000000 ||
              av_gettime_relative() - target > 2000000) {
            first_pts = pts;
            clock_start = av_gettime_relative();
            target = clock_start;
          }
          while (target > av_gettime_relative() && !control(user, 0))
            av_usleep(5000);
        }
        video(user, pixels, out_width, out_height, out_width * 4, pts);
        if (input_kind == 3) {
          result = 0;
          goto done;
        }
      } else {
        int volume = control(user, 1);
        if (volume > 0) {
          if (!audio) {
            pa_sample_spec spec = {PA_SAMPLE_S16LE, 48000, 2};
            pa_buffer_attr attr = {.maxlength = 48000,
                                   .tlength = 9600,
                                   .prebuf = 0,
                                   .minreq = 1920,
                                   .fragsize = 0};
            int error;
            audio = pa_simple_new(NULL, "CCTV View", PA_STREAM_PLAYBACK, NULL,
                                  "Câmera", &spec, NULL, &attr, &error);
          }
          if (!resample) {
            AVChannelLayout stereo = AV_CHANNEL_LAYOUT_STEREO;
            if (swr_alloc_set_opts2(&resample, &stereo, AV_SAMPLE_FMT_S16,
                                    48000, &frame->ch_layout, frame->format,
                                    frame->sample_rate, 0, NULL) < 0 ||
                swr_init(resample) < 0) {
              swr_free(&resample);
            }
          }
          if (audio && resample) {
            int count = swr_get_out_samples(resample, frame->nb_samples);
            uint8_t *samples = NULL;
            if (count > 0 && count < 192000 &&
                av_samples_alloc(&samples, NULL, 2, count, AV_SAMPLE_FMT_S16,
                                 0) >= 0) {
              int n = swr_convert(resample, &samples, count,
                                  (const uint8_t **)frame->extended_data,
                                  frame->nb_samples);
              if (n > 0) {
                int16_t *s = (int16_t *)samples;
                for (int j = 0; j < n * 2; j++)
                  s[j] = (int16_t)(s[j] * volume / 100);
                int error;
                if (pa_simple_write(audio, samples, (size_t)n * 4, &error) <
                    0) {
                  pa_simple_free(audio);
                  audio = NULL;
                } else {
                  control(user, 100 + n * 4);
                }
              }
              av_freep(&samples);
            }
          }
        } else if (audio) {
          int error;
          pa_simple_flush(audio, &error);
          pa_simple_free(audio);
          audio = NULL;
        }
      }
      av_frame_unref(frame);
    }
    if (result == AVERROR(EAGAIN))
      result = 0;
  }
done:
  if (audio)
    pa_simple_free(audio);
  swr_free(&resample);
  sws_freeContext(scale);
  av_free(pixels);
  av_frame_free(&software);
  av_frame_free(&frame);
  av_packet_free(&packet);
  avcodec_free_context(&vd);
  avcodec_free_context(&ad);
  avformat_close_input(&fmt);
  return result;
}
void cctv_media_init(void) {
  av_log_set_level(AV_LOG_QUIET);
  avformat_network_init();
}
