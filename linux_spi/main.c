#include <inttypes.h>
#include <stdio.h>
#include <fcntl.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <linux/ioctl.h>
#include <linux/spi/spidev.h>
#include <stdlib.h>

const char *default_device = "/dev/spidev0.0";

void pabort(const char *s)
{
	perror(s);
	abort();
}


int16_t change_endianness_16(int16_t val) {
  return (val << 8) |          // left-shift always fills with zeros
    ((val >> 8) & 0x00ff); // right-shift sign-extends, so force to zero
}

ssize_t write_16(int fd, uint16_t value) {
  /* uint16_t with_correct_endianness = change_endianness_16(value); */
  /* return write(fd, &with_correct_endianness, sizeof(with_correct_endianness)); */
  return write(fd, &value, sizeof(value));
}

void fade_test(int fd) {
  uint16_t buf = 0xAAAA;

  uint16_t step = 1;
  uint32_t sleep_us = 0;

  printf("fade in...\n");
  fflush(stdout);

  uint16_t max = 0x1000;

  for(buf = 0; buf < (max - step); buf += step) {
    write_16(fd, buf);
    if (sleep_us > 0)
      usleep(sleep_us);
  }

  printf("fade out...\n");
  fflush(stdout);

  for(buf = max; buf > step; buf -= step) {
    write_16(fd, buf);
    if (sleep_us > 0)
      usleep(sleep_us);
  }

  printf("fade end.\n");
  fflush(stdout);

  /* for(uint32_t i = 0; i < 0xFFFF; i++) { */
  /*   write_16(fd, buf); */
  /*   if (sleep_us > 0) */
  /*     usleep(sleep_us); */
  /* } */

  /* printf("end test.\n"); */
  /* fflush(stdout); */
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

  int fd = open(dev_path, O_RDWR);

  if (fd == -1) {
    fprintf(stderr, "Could not open device: %s\n", strerror(errno));
  }

  uint32_t speed = 1000000;
  int ret = ioctl(fd, SPI_IOC_WR_MAX_SPEED_HZ, &speed);
	if (ret == -1)
		pabort("can't set max speed hz");


  uint8_t bits = 8;
  ret = ioctl(fd, SPI_IOC_WR_BITS_PER_WORD, &bits);
	if (ret == -1)
		pabort("can't set bits per word");

  uint8_t lsb = 0;
  ret = ioctl(fd, SPI_IOC_WR_LSB_FIRST, &lsb);
	if (ret == -1)
		pabort("can't set lsb");


  /* char buf[] = "hello kitty blah blah blah 123"; */
  // char buf[] = { 0xff, 0x00 };
  /* size_t buf_size = sizeof(buf); */
  // ssize_t written_bytes = write(fd, buf, buf_size);

  fade_test(fd);

  /* if (written_bytes == -1) { */
  /*   fprintf(stderr, "Could not write to file: %s\n", strerror(errno)); */
  /*   return(1); */
  /* } */

  /* fprintf(stderr, "written %ld bytes out of %lu\n", written_bytes, buf_size); */

  return 0;
}
