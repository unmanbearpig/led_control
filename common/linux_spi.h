#pragma once

#include <inttypes.h>
#include <errno.h>
#include <fcntl.h>
#include <sys/ioctl.h>
#include <linux/ioctl.h>
#include <linux/spi/spidev.h>
#include <string.h>
#include "linux_util.h"

#define SPI_DEFAULT_FREQUENCY 16000000

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
    speed = SPI_DEFAULT_FREQUENCY;
  }

  int fd = open(dev_path, O_RDONLY);

  if (fd == -1) {
    fprintf(stderr, "Could not open device: %s\n", strerror(errno));
  }


	if (ioctl(fd, SPI_IOC_WR_MAX_SPEED_HZ, &speed))
		pabort("can't set max wr speed hz");

  if (ioctl(fd, SPI_IOC_RD_MAX_SPEED_HZ, &speed))
		pabort("can't set max rd speed hz");

  uint8_t bits = 8;

	if (ioctl(fd, SPI_IOC_WR_BITS_PER_WORD, &bits))
		pabort("can't set bits per word");

  uint8_t lsb = 0;
	if (ioctl(fd, SPI_IOC_WR_LSB_FIRST, &lsb))
		pabort("can't set lsb");

  return fd;
}
