#include <inttypes.h>
#include <unistd.h>
#include <stdlib.h>
#include <stdio.h>
#include <getopt.h>
#include <string.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include "../common/protocol.h"
#include "../common/protocol_debug.h"
#include "../common/secrets.h"
#include "../common/protocol_udp.h"

int parse_args(int argc, char *argv[], char **host, int *port, uint16_t *type) {
  int argid = 0;

  struct option opts[] =
    {
     { .name = "host", .has_arg = required_argument, .flag = &argid, .val = 'h' },
     { .name = "port", .has_arg = required_argument, .flag = &argid, .val = 'p' },
     { .name = "type", .has_arg = required_argument, .flag = &argid, .val = 't' },
     { 0, 0, 0, 0 }
    };

  int longindex = 0;
  int ch = 0;

  while( (ch = getopt_long(argc, argv, "h:p:t:", opts, &longindex)) != -1 ) {
    char *strtol_endptr = 0;

    switch(argid) {
    case 'h':
      *host = optarg;
      break;
    case 'p':
      *port = atoi(optarg);
      break;
    case 't':
      *type = strtol(optarg, &strtol_endptr, 0);

      if (strtol_endptr == optarg) {
        fprintf(stderr, "first argument should be a value, but it is: %s\n", optarg);
        exit(1);
      }

      break;
    default:
      return 1; // whatever
    }
  }

  return(1);
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    fprintf(stderr, "usage: set_value VALUE [--type TYPE] [--host HOST] [--port PORT]\n");
    return(1);
  }

  char *strtol_endptr = 0;
  uint16_t value = strtol(argv[1], &strtol_endptr, 0);

  if (strtol_endptr == argv[1]) {
    fprintf(stderr, "first argument should be a value, but it is: %s\n", argv[1]);
    exit(1);
  }

  char *dst_host = "192.168.0.102";
  int dst_port = DEFAULT_UDP_PORT;
  uint16_t type = LED_VALUES_MESSAGE_MAGIC;

  parse_args(argc, argv, &dst_host, &dst_port, &type);

  int sock = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
  if (sock == -1) {
    fprintf(stderr, "socket failed\n");
    exit(1);
  }

  struct sockaddr_in sa;
  memset(&sa, 0, sizeof(sa));

  sa.sin_family = AF_INET;
  sa.sin_addr.s_addr = inet_addr(dst_host);
  sa.sin_port = htons(dst_port);


  LedValuesMessage msg;
  memset(&msg, 0, sizeof(msg));
  msg.magic = type;
  msg.led_values[0] = value;
  msg.led_values[1] = value;
  msg.led_values[2] = value;
  msg.led_values[3] = value;

  print_msg(&msg, ">");
  send_msg(sock, &sa, &msg);


  LedValuesMessage recv_msg;
  memset(&recv_msg, 0, sizeof(recv_msg));
  int recsize = recvfrom(sock, &recv_msg, sizeof(recv_msg), 0, NULL, NULL);

  if (recsize == -1) {
    fprintf(stderr, "recsize == -1\n");
    return(1);
  }

  print_msg(&recv_msg, "<");
}
