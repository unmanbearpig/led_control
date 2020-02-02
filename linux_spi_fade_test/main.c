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

/* ssize_t write_msg(int fd, LedValuesMessage *msg) { */
/*   char buf[sizeof(*msg)] = { 0 }; */

/*   memcpy(buf, msg, sizeof(buf)); */

/*   /\* printf("%x %x %x %x %x\n", buf[0], buf[1], buf[2], buf[3], buf[4]); *\/ */
/*   /\* fflush(stdout); *\/ */
/*   ssize_t bytes_written = write(fd, buf, sizeof(buf)); */

/*   printf("> %ld/%lu:  %04x %04x %04x %04x %04x\n", bytes_written, sizeof(*msg), buf[0], buf[1], buf[2], buf[3], buf[4]); */

/*   ssize_t bytes_read = read(fd, buf, sizeof(buf)); */
/*   printf("< %ld/%lu:  %04x %04x %04x %04x %04x\n", bytes_read, sizeof(*msg), buf[0], buf[1], buf[2], buf[3], buf[4]); */
/*   fflush(stdout); */

/*   return bytes_written; */
/* } */

ssize_t write_msg(int fd, LedValuesMessage *msg) {
  const size_t len = sizeof(*msg);
  uint16_t tx_buf[sizeof(*msg)] = { 0 };
  uint16_t rx_buf[sizeof(*msg)] = { 0 };

  memcpy(tx_buf, msg, len);
  memset(rx_buf, 0, sizeof(rx_buf));

  printf(">    %ld/%lu:  %04x %04x %04x %04x %04x\n", len, len, tx_buf[0], tx_buf[1], tx_buf[2], tx_buf[3], tx_buf[4]);

  struct spi_ioc_transfer xfer;

  memset(&xfer, 0, sizeof xfer);
  xfer.tx_buf = (unsigned long)tx_buf;
  xfer.rx_buf = (unsigned long)rx_buf;
  xfer.len = len;

  /* xfer[1].rx_buf = (unsigned long) buf; */
  /* xfer[1].len = len; */

  int status = ioctl(fd, SPI_IOC_MESSAGE(1), &xfer);

  printf("< %02d %ld/%lu:  %04x %04x %04x %04x %04x\n", status, len, len, rx_buf[0], rx_buf[1], rx_buf[2], rx_buf[3], rx_buf[4]);

  return 0;
}

void fade_test(int fd) {
  uint16_t buf = 0xAAAA;

  uint16_t step = 0x18;
  uint32_t sleep_us = 0;

  printf("fade in...\n");
  fflush(stdout);

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

  printf("fade out...\n");
  fflush(stdout);

  for(buf = max; buf > step; buf -= step) {
    msg.led1_value = buf;
    msg.led2_value = buf;
    msg.led3_value = buf;
    msg.led4_value = buf;

    write_msg(fd, &msg);
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

  uint32_t speed = 10000000;
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
