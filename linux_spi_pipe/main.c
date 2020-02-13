#include <inttypes.h>
#include <errno.h>
#include <unistd.h>
#include <string.h>
#include "../common/linux_spi.h"
#include "../common/linux_util.h"


const char *default_device = "/dev/spidev0.0";

/* ssize_t write_bytes(int fd, void *buf, size_t len) { */
/*   size_t bytes_written = 0; */

/*   while(bytes_written < len) { */
/*     ssize_t tmp_bytes_written = write(fd, buf + bytes_written, len - bytes_written); */
/*     if (tmp_bytes_written < 0) { */
/*       return tmp_bytes_written; */
/*     } */

/*     bytes_written += tmp_bytes_written; */
/*   } */

/*   return bytes_written; */
/* } */


/* ssize_t write_msg(int fd, LedValuesMessage *msg) { */
/*   return write_bytes(fd, msg, sizeof(*msg)); */
/* } */

/* ssize_t read_msg(int fd, LedValuesMessage *msg) { */
/*   size_t bytes_read = 0; */

/*   char buf[sizeof(*msg)] = { 0 }; */

/*   while(bytes_read < sizeof(*msg)) { */
/*     bytes_read += read(fd, */
/*                        buf + bytes_read, */
/*                        sizeof(*msg) - bytes_read); */
/*   } */

/*   memcpy((void *) msg, buf, sizeof(*msg)); */

/*   fprintf(stderr, "> %02X%02X %02X%02X %02X%02X %02X%02X %02X%02X\n", */
/*           buf[0], buf[1], buf[2], buf[3], buf[4], */
/*           buf[5], buf[6], buf[7], buf[8], buf[9]); */

/*   return bytes_read; */
/* } */

/* int main(int argc, char *argv[]) { */
/*   const char *dev_path = 0; */

/*   if (argc == 2) { */
/*     dev_path = argv[1]; */
/*   } else if (argc == 1) { */
/*     dev_path = default_device; */
/*   } else { */
/*     fprintf(stderr, "Too many arguments. expect spi device path\n"); */
/*     return 1; */
/*   } */

/*   int fd = open(dev_path, O_RDWR); */

/*   if (fd == -1) { */
/*     fprintf(stderr, "Could not open device: %s\n", strerror(errno)); */
/*   } */

/*   uint32_t speed = 1000000; */
/*   int ret = ioctl(fd, SPI_IOC_WR_MAX_SPEED_HZ, &speed); */
/* 	if (ret == -1) */
/* 		pabort("can't set max speed hz"); */


/*   uint8_t bits = 8; */
/*   ret = ioctl(fd, SPI_IOC_WR_BITS_PER_WORD, &bits); */
/* 	if (ret == -1) */
/* 		pabort("can't set bits per word"); */

/*   uint8_t lsb = 0; */
/*   ret = ioctl(fd, SPI_IOC_WR_LSB_FIRST, &lsb); */
/* 	if (ret == -1) */
/* 		pabort("can't set lsb"); */

/*   LedValuesMessage msg = { 0 }; */

/*   for(;;) { */
/*     ssize_t bytes_read = read_msg(STDIN_FILENO, &msg); */

/*     if (bytes_read != sizeof(msg)) { */
/*       fprintf(stderr, "expected %ld bytes, but read only %ld\n", sizeof(msg), bytes_read); */
/*       return 1; */
/*     } */

/*     write_msg(fd, &msg); */
/*   } */
/* } */




int main(int argc, char *argv[]) {
  int spi = try_open_spi("/dev/spidev0.0", 0);

  const size_t len = 10;

  void *tx_buf = malloc(len);
  if ((size_t)tx_buf == ENOMEM) {
    pabort("lol no mem!");
  }

  void *rx_buf = malloc(len);
  if ((size_t)rx_buf == ENOMEM) {
    pabort("lol no mem!");
  }

  memset(tx_buf, 0, len);
  memset(rx_buf, 0, len);

  while(1) {
    if (read_all(STDIN_FILENO, tx_buf, len) < 0) {
      pabort("Could not read input lol");
    }

    int xfer_bytes = spi_xfer_bytes(spi, tx_buf, rx_buf, len);

    fprintf(stdout, "%d.", xfer_bytes);
  }
}
