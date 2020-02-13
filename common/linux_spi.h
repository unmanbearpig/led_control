#pragma once

#include <inttypes.h>
#include <errno.h>
#include <fcntl.h>
#include <sys/ioctl.h>
#include <linux/ioctl.h>
#include <linux/spi/spidev.h>
#include <string.h>
#include "linux_util.h"

const uint32_t spi_default_speed = 10000000;

// transmits tx_buf to SPI while receiving from SPI to rx_buf
// tx_buf and rx_buf can probably be the same (haven't checked)
int spi_xfer_bytes(int fd, void *tx_buf, void *rx_buf, size_t len) {
  struct spi_ioc_transfer xfer;
  memset(&xfer, 0, sizeof xfer);
  xfer.tx_buf = (unsigned long)tx_buf;
  xfer.rx_buf = (unsigned long)rx_buf;
  xfer.len = len;

  int status = ioctl(fd, SPI_IOC_MESSAGE(1), &xfer);

  return status;
}

int try_open_spi(const char *dev_path, uint32_t speed) {
  if (speed == 0) {
    speed = spi_default_speed;
  }

  int fd = open(dev_path, O_RDWR);

  if (fd == -1) {
    fprintf(stderr, "Could not open device: %s\n", strerror(errno));
  }

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

  return fd;
}
