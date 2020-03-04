#include <inttypes.h>
#include <stdio.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
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
  float step = 0.02;

  uint32_t sleep_us = 0;

  // uint16_t max = 0xFFFF;

  LedValuesMessage msg =
    {
     .magic = led_values_message_magic,
     .led1_value = 0,
     .led2_value = 0,
     .led3_value = 0,
     .led4_value = 0,
    };

  fprintf(stderr, "fade in\n");
  fflush(stderr);

  for(; buf < (1.0 - 0.00002); buf *= (1.0 + step)) {
    uint16_t val = buf * 0xFFFF;

    if (buf >= 1.0) {
      val = 0xFFFF;
    }

    set_all_msg_values(&msg, val);

    write_msg(fd, &msg);
    if(sleep_us > 0) {
      usleep(sleep_us);
    }
  }

  fprintf(stderr, "fade out\n");
  fflush(stderr);

  for(; buf > 0.00002; buf *= (1.0 - step)) {
    uint16_t val = buf * 0xFFFF;

    if (buf >= 1.0) {
      val = 0xFFFF;
    }
    set_all_msg_values(&msg, val);

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
