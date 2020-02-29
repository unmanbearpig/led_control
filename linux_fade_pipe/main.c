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
  uint16_t buf = 0xAAAA;

  uint16_t step = 1;
  uint32_t sleep_us = 100;

  fprintf(stderr, "fade in...\n");
  fflush(stderr);

  uint16_t max = 0xFFFF;

  LedValuesMessage msg =
    {
     .magic = led_values_message_magic,
     .led1_value = 0,
     .led2_value = 0,
     .led3_value = 0,
     .led4_value = 0,
    };

  for(buf = 0; buf < (max - step); buf += step) {
    msg.led1_value = buf;
    msg.led2_value = buf;
    msg.led3_value = buf;
    msg.led4_value = buf;

    write_msg(fd, &msg);
    if (sleep_us > 0)
      usleep(sleep_us);
  }

  fprintf(stderr, "fade out...\n");
  fflush(stderr);

  for(buf = max; buf > step; buf -= step) {
    msg.led1_value = buf;
    msg.led2_value = buf;
    msg.led3_value = buf;
    msg.led4_value = buf;

    write_msg(fd, &msg);
    if (sleep_us > 0)
      usleep(sleep_us);
  }

  fprintf(stderr, "fade end.\n");
  fflush(stderr);

  /* for(uint32_t i = 0; i < 0xFFFF; i++) { */
  /*   write_16(fd, buf); */
  /*   if (sleep_us > 0) */
  /*     usleep(sleep_us); */
  /* } */

  /* printf("end test.\n"); */
  /* fflush(stdout); */
}

int main(int argc, char *argv[]) {
  fade_test(1);
}
