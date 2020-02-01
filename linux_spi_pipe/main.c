#include <inttypes.h>
#include <stdio.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
#include <linux/spi/spidev.h>
#include <sys/ioctl.h>
#include <linux/ioctl.h>
#include <stdlib.h>
#include <string.h>
#include "../common_headers/structs.h"

const char *default_device = "/dev/spidev0.0";

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
  ssize_t bytes_written = write(fd, msg, sizeof(*msg));
  // fflush(fd);

  return bytes_written;
}

ssize_t read_msg(int fd, LedValuesMessage *msg) {
  size_t bytes_read = 0;

  char buf[sizeof(*msg)] = { 0 };

  while(bytes_read < sizeof(*msg)) {
    bytes_read += read(fd,
                       buf + bytes_read,
                       sizeof(*msg) - bytes_read);
  }

  memcpy((void *) msg, buf, sizeof(*msg));

  fprintf(stderr, "> %02X %02X %02X %02X %02X\n",
          buf[0], buf[1], buf[2], buf[3], buf[4]);

  return bytes_read;
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

  uint32_t speed = 100000;
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

  LedValuesMessage msg = { 0 };

  for(;;) {
    ssize_t bytes_read = read_msg(STDIN_FILENO, &msg);

    if (bytes_read != sizeof(msg)) {
      fprintf(stderr, "expected %ld bytes, but read only %ld\n", sizeof(msg), bytes_read);
      return 1;
    }

    write_msg(fd, &msg);
  }
}
