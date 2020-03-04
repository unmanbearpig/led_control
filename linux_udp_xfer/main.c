#include <inttypes.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <getopt.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include "../common/protocol.h"
#include "../common/protocol_debug.h"
#include "../common/secrets.h"

void print_usage() {
  fprintf(stderr, "\
--host HOST\n\
--port PORT\n\
");
}

int parse_args(int argc, char *argv[], char **host, int *port) {
  int argid = 0;
  struct option opts[] =
    {
     { .name = "host", .has_arg = required_argument, .flag = &argid, .val = 'h' },
     { .name = "port", .has_arg = required_argument, .flag = &argid, .val = 'p' },
     { 0, 0, 0, 0 }
    };

  int longindex = 0;
  int ch = 0;

  while( (ch = getopt_long(argc, argv, "s:g:c:t:r:n:v", opts, &longindex)) != -1 ) {
    switch(argid) {
    case 'h':
      *host = optarg;
      break;
    case 'p':
      *port = atoi(optarg);
      break;
    default:
      fprintf(stderr, "Invalid argument\n");
      print_usage();
      return(0);
      break;
    }
  }

  return(1);
}

int main(int argc, char *argv[]) {
  char *host = 0;
  int port = 0;

#ifdef DEFAULT_UDP_PORT
  port = DEFAULT_UDP_PORT;
#endif

  if (!parse_args(argc, argv, &host, &port)) {
    fprintf(stderr, "invalid arguments\n");
    print_usage();
    exit(1);
  }

  if (host == 0) {
    fprintf(stderr, "host is not defined\n");
    print_usage();
    exit(1);
  }

  if (port == 0) {
    fprintf(stderr, "port is not defined\n");
    print_usage();
    exit(1);
  }

  int sock = 0;

  sock = socket(AF_INET, SOCK_DGRAM | SOCK_NONBLOCK, IPPROTO_UDP);
  if (sock == -1) {
    fprintf(stderr, "socket failed\n");
    exit(1);
  }

  struct sockaddr_in sa;
  memset(&sa, 0, sizeof(sa));

  sa.sin_family = AF_INET;
  sa.sin_addr.s_addr = inet_addr(host);
  sa.sin_port = htons(port);

  uint8_t buf[sizeof(LedValuesMessage)];

  memset(buf, 0, sizeof(buf));

  int sleep_us = 5000;

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
