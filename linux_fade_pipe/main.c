#include <inttypes.h>
#include <stdio.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
#include <linux/spi/spidev.h>
#include <stdlib.h>
#include "../common_headers/structs.h"

void pabort(const char *s)
{
	perror(s);
	abort();
}

ssize_t write_msg(int fd, LedValuesMessage *msg) {
  /* char buf[sizeof(*msg)] = { 0 }; */
  /* memcpy(buf, msg, sizeof(buf)); */
  /* printf("%x %x %x %x %x\n", buf[0], buf[1], buf[2], buf[3], buf[4]); */
  /* fflush(stdout); */
  ssize_t bytes_written = write(fd, msg, sizeof(msg));
  // fflush(fd);

  return bytes_written;
}

void fade_test(int fd) {
  uint16_t buf = 0xAAAA;

  uint16_t step = 1;
  uint32_t sleep_us = 20;

  fprintf(stderr, "fade in...\n");
  fflush(stderr);

  uint16_t max = 0x1000;

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
