#include "linux_spi.h"
#include "protocol.h"

int xfer_msg(int fd, LedValuesMessage *msg, int verbose) {
  const size_t len = sizeof(*msg);

  void *tx_buf = (void *)msg;
  uint16_t rx_buf[sizeof(*msg)] = { 0 };

  if (verbose) {
    printf(">    %lu:  %04x %04x %04x %04x %04x\n", len, ((uint16_t *)tx_buf)[0], ((uint16_t *)tx_buf)[1], ((uint16_t *)tx_buf)[2], ((uint16_t *)tx_buf)[3], ((uint16_t *)tx_buf)[4]);
  }

  int status = spi_xfer_bytes(fd, tx_buf, rx_buf, sizeof(*msg));

  if (verbose) {
    printf("< %02d %lu:  %04x %04x %04x %04x %04x\n", status, len, rx_buf[0], rx_buf[1], rx_buf[2], rx_buf[3], rx_buf[4]);
  }

  return status;
}

int xfer_msg_2(int fd, LedValuesMessage *send_msg, LedValuesMessage *recv_msg, int verbose) {
  const size_t len = sizeof(*send_msg);

  void *tx_buf = (void *)send_msg;
  uint16_t *rx_buf = (uint16_t *)recv_msg;

  if (verbose) {
    printf(">    %lu:  %04x %04x %04x %04x %04x\n", len, ((uint16_t *)tx_buf)[0], ((uint16_t *)tx_buf)[1], ((uint16_t *)tx_buf)[2], ((uint16_t *)tx_buf)[3], ((uint16_t *)tx_buf)[4]);
  }

  int status = spi_xfer_bytes(fd, tx_buf, rx_buf, sizeof(*send_msg));

  if (verbose) {
    printf("< %02d %lu:  %04x %04x %04x %04x %04x\n", status, len, rx_buf[0], rx_buf[1], rx_buf[2], rx_buf[3], rx_buf[4]);
  }

  return status;
}
