#include <inttypes.h>
#include <stdio.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <linux/ioctl.h>
#include <linux/spi/spidev.h>
#include <stdlib.h>
#include <string.h>
#include "../common/protocol.h"
#include "../common/linux_spi.h"
#include "../common/linux_spi_protocol.h"

const char *default_device = "/dev/spidev0.0";

void fade_test(int fd) {
  uint16_t buf = 0xAAAA;

  uint16_t step = 0x01;
  uint32_t sleep_us = 100;

  printf("fade in...\n");
  fflush(stdout);

  uint16_t max = 0x0F00;

  LedValuesMessage msg =
    {
     .magic = LED_VALUES_MESSAGE_MAGIC,
     .led_values = { 0, 0, 0, 0 }
    };

  for(buf = 0; buf < (max - step); buf += step) {
    msg.led_values[0] = buf;
    msg.led_values[1] = buf;
    msg.led_values[2] = buf;
    msg.led_values[3] = buf;

    xfer_msg(fd, &msg, 1);
    if (sleep_us > 0)
      usleep(sleep_us);
  }

  printf("fade out...\n");
  fflush(stdout);

  for(buf = max; buf > step; buf -= step) {
    msg.led_values[0] = buf;
    msg.led_values[1] = buf;
    msg.led_values[2] = buf;
    msg.led_values[3] = buf;

    xfer_msg(fd, &msg, 1);
    if (sleep_us > 0)
      usleep(sleep_us);
  }

  printf("fade end.\n");
  fflush(stdout);
}

int main(int argc, char *argv[]) {
  const char *dev_path = 0;

  if (argc == 2) {
    dev_path = argv[1];
  } else if (argc == 1) {
    dev_path = default_device;
  } else {
    fprintf(stderr, "Too many arguments. expect spi device path\n");
    return 1;
  }
  int fd = try_open_spi(dev_path, 0);

  fade_test(fd);

  return 0;
}
