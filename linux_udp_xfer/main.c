#include <inttypes.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include "../common/protocol.h"
#include "../common/protocol_debug.h"
#include "../common/secrets.h"

int main(int argc, char *argv[]) {
  int sock = 0;

  char *dst_host = "192.168.0.105";
  int dst_port = DEFAULT_UDP_PORT;

  sock = socket(AF_INET, SOCK_DGRAM | SOCK_NONBLOCK, IPPROTO_UDP);
  if (sock == -1) {
    fprintf(stderr, "socket failed\n");
    exit(1);
  }

  struct sockaddr_in sa;
  memset(&sa, 0, sizeof(sa));

  sa.sin_family = AF_INET;
  sa.sin_addr.s_addr = inet_addr(dst_host);
  sa.sin_port = htons(dst_port);

  uint8_t buf[sizeof(LedValuesMessage)];

  memset(buf, 0, sizeof(buf));

  int sleep_us = 10000;

  for(;;) {
    int bytes_read = read(STDIN_FILENO, buf, sizeof(buf));

    if (bytes_read == sizeof(buf)) {
      sendto(sock, buf, sizeof(buf), 0, (struct sockaddr *)&sa, sizeof(sa));
    }

    ssize_t bytes_received = recv(sock, buf, sizeof(buf), 0);

    if (bytes_received != -1) {
      if (bytes_received != sizeof(buf)) {
        fprintf(stderr, "received %ld bytes instead of %lu\n", bytes_received, (unsigned long)sizeof(buf));
      }
      write(STDOUT_FILENO, buf, sizeof(buf));
    }

    usleep(sleep_us);
  }

  return 0;
}
