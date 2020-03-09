#include <inttypes.h>
#include <stdio.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
#include <string.h>
#include <linux/spi/spidev.h>
#include <stdlib.h>
#include "../common/protocol.h"

void pabort(const char *s)
{
	perror(s);
	abort();
}

ssize_t write_bytes(int fd, void *buf, size_t len) {
  size_t bytes_written = 0;

  while(bytes_written < len) {
    ssize_t tmp_bytes_written = write(fd, buf + bytes_written, len - bytes_written);
    if (tmp_bytes_written < 0) {
      return tmp_bytes_written;
    }

    bytes_written += tmp_bytes_written;
  }

  return bytes_written;
}

ssize_t write_msg(int fd, LedValuesMessage *msg) {
  return write_bytes(fd, msg, sizeof(*msg));
}

void fade_test(int fd) {
  float buf = 0.00001;
  uint32_t sleep_us = 0;

  LedValuesMessage msg;
  memset(&msg, 0, sizeof(msg));
  msg.magic = LED_VALUES_MESSAGE_MAGIC;
  msg.type = LED_WRITE | LED_READ;
  msg.payload.data.flags = LED_VALUES_FLAG_FLOAT;
  msg.payload.data.amount = 1.0;

  fprintf(stderr, "fade in\n");
  fflush(stderr);

  float max = 0.3;
  float min = 0.0;
  float step = 50.0 / 65535.0;

  for(; buf < max; buf += step) {
    set_all_msg_float_values(&msg, buf);
    write_msg(fd, &msg);
    if(sleep_us > 0) {
      usleep(sleep_us);
    }
  }

  fprintf(stderr, "fade out\n");
  fflush(stderr);

  for(; buf > min; buf -= step) {
    set_all_msg_float_values(&msg, buf);
    write_msg(fd, &msg);
    if(sleep_us > 0) {
      usleep(sleep_us);
    }
  }

  fprintf(stderr, "fade end.\n");
  fflush(stderr);
}

int main(int argc, char *argv[]) {
  fade_test(1);
  return(0);
}
