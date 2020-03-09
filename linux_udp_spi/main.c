#include <inttypes.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include "../common/protocol.h"
#include "../common/linux_spi_protocol.h"
#include "../common/secrets.h"

int main(int argc, char *argv[]) {
  int sock = 0;

  char *spi_path = "/dev/spidev0.0";
  int spi_fd = try_open_spi(spi_path, 0);

  if ((sock = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP)) == -1)	{
    fprintf(stderr, "socket failed\n");
    exit(1);
	}

  struct sockaddr_in sin;

  memset(&sin, 0, sizeof(sin));

  sin.sin_family = AF_INET;
  sin.sin_addr.s_addr = htonl(INADDR_ANY);
  sin.sin_port = htons(DEFAULT_UDP_PORT);

  if (bind(sock, (struct sockaddr *)&sin, sizeof(sin)) == -1) {
    fprintf(stderr, "bind failed\n");
    exit(1);
  }

  LedValuesMessage msg;
  memset(&msg, 0, sizeof(msg));

  struct sockaddr_storage peer_addr;
  unsigned int peer_addr_len = 0;
  memset(&peer_addr, 0, sizeof(peer_addr));

  LedValuesMessage recv_msg;
  memset(&recv_msg, 0xEE, sizeof(recv_msg));

  for (;;) {
    int recsize = recvfrom(sock, &msg, sizeof(msg), 0, (struct sockaddr *)&peer_addr, &peer_addr_len);

    if (recsize != -1) {
      if (sock < 0) {
        fprintf(stderr, "accept failed\n");
        exit(1);
      }
      // read(sock, &msg, sizeof(msg));
      xfer_msg_2(spi_fd, &msg, &recv_msg, 0);
      // write(sock, &msg, sizeof(msg));
      sendto(sock, &recv_msg, sizeof(recv_msg), 0, (struct sockaddr *)&peer_addr, peer_addr_len);

      // print_msg(&msg);
    }
    // usleep(sleep_us);
  }

  return 0;
}
