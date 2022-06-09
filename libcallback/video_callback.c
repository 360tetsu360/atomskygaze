#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <stdint.h>
#include <fcntl.h>
#include <linux/videodev2.h>
#include <sys/ioctl.h>
#include <sys/time.h>
#include <pthread.h>

struct frames_st {
  uint32_t *buf;
  size_t length;
};
typedef int (* framecb)(struct frames_st *);
static int (*real_local_sdk_video_set_encode_frame_callback)(int ch, void *callback);
static int video0_encode_capture(struct frames_st *frames);
static int video1_encode_capture(struct frames_st *frames);

struct video_capture_st {
  framecb callback;
  framecb capture;
  int enable;
  int initialized;
  int width;
  int height;
  const char *device;
  int fd;
};
static struct video_capture_st video_capture[] = {
  {
    .callback = NULL,
    .capture = video0_encode_capture,
    .enable = 0,
    .initialized = 0,
    .width = 1920,
    .height = 1080,
    .device = "/dev/video0",
    .fd = -1,
  },
  {
    .callback = NULL,
    .capture = video1_encode_capture,
    .enable = 0,
    .initialized = 0,
    .width = 640,
    .height = 360,
    .device = "/dev/video1",
    .fd = -1,
  },
};

char *VideoCapture(int fd, char *tokenPtr) {

  char *p = strtok_r(NULL, " \t\r\n", &tokenPtr);
  int ch = 0;
  if(p && (!strcmp(p, "0") || !strcmp(p, "1"))) {
    ch = atoi(p);
    p = strtok_r(NULL, " \t\r\n", &tokenPtr);
  }
  if(!p) return video_capture[ch].enable ? "on" : "off";
  if(!strcmp(p, "on")) {
    video_capture[ch].enable = 1;
    printf("[command] video %d capute on\n", ch);
    return "ok";
  }
  if(!strcmp(p, "off")) {
    video_capture[ch].enable = 0;
    printf("[command] video %d capute off\n", ch);
    return "ok";
  }
  return "error";
}

static int video_encode_capture(int ch, struct frames_st *frames) {

  if(!video_capture[ch].initialized) {
    video_capture[ch].initialized = 1;
    int err;
    video_capture[ch].fd = open(video_capture[ch].device, O_WRONLY, 0777);
    if(video_capture[ch].fd < 0) fprintf(stderr, "Failed to open V4L2 device: %s\n", video_capture[ch].device);
    fprintf(stderr, "Opening V4L2 device: %s %d\n", video_capture[ch].device, video_capture[ch].fd);
    struct v4l2_format vid_format;
    memset(&vid_format, 0, sizeof(vid_format));
    vid_format.type = V4L2_BUF_TYPE_VIDEO_OUTPUT;
    vid_format.fmt.pix.width = video_capture[ch].width;
    vid_format.fmt.pix.height = video_capture[ch].height;
    vid_format.fmt.pix.pixelformat = V4L2_PIX_FMT_H264;
    vid_format.fmt.pix.sizeimage = 0;
    vid_format.fmt.pix.field = V4L2_FIELD_NONE;
    vid_format.fmt.pix.bytesperline = 0;
    vid_format.fmt.pix.colorspace = V4L2_PIX_FMT_YUV420;
    err = ioctl(video_capture[ch].fd, VIDIOC_S_FMT, &vid_format);
    if(err < 0) fprintf(stderr, "Unable to set V4L2 %s format: %d\n", video_capture[ch].device, err);
    err = ioctl(video_capture[ch].fd, VIDIOC_STREAMON, &vid_format);
    if(err < 0) fprintf(stderr, "Unable to perform VIDIOC_STREAMON %s: %d\n", video_capture[ch].device, err);
  }

  if((video_capture[ch].fd >= 0) && video_capture[ch].enable) {
    uint32_t *buf = frames->buf;
    int size = write(video_capture[ch].fd, frames->buf, frames->length);
    if(size != frames->length) fprintf(stderr,"Stream write error %s: %s\n", video_capture[ch].device, size);
  }
  return (video_capture[ch].callback)(frames);
}

static int video0_encode_capture(struct frames_st *frames) {
  return video_encode_capture(0, frames);
}

static int video1_encode_capture(struct frames_st *frames) {
  return video_encode_capture(1, frames);
}

int local_sdk_video_set_encode_frame_callback(int ch, void *callback) {

  fprintf(stderr, "local_sdk_video_set_encode_frame_callback streamChId=%d, callback=0x%x\n", ch, callback);
  if((ch == 0) || (ch == 1)) {
    video_capture[ch].callback = callback;
    fprintf(stderr,"enc func injection save video_encode_cb=0x%x\n", video_capture[ch].callback);
    callback = video_capture[ch].capture;
  }
  return real_local_sdk_video_set_encode_frame_callback(ch, callback);
}

static void __attribute ((constructor)) video_callback_init(void) {

  real_local_sdk_video_set_encode_frame_callback = dlsym(dlopen("/system/lib/liblocalsdk.so", RTLD_LAZY), "local_sdk_video_set_encode_frame_callback");
}
