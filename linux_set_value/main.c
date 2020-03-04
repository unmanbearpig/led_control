#include <inttypes.h>
#include <unistd.h>
#include <stdlib.h>
#include <stdio.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include "../common/protocol.h"
#include "../common/protocol_debug.h"
#include "../common/secrets.h"
#include "../common/args.h"

int send_msg(int sock, struct sockaddr_in *sa, LedValuesMessage *msg) {
  print_msg(msg);
  return sendto(sock, msg, sizeof(*msg), 0, (struct sockaddr *)sa, sizeof(*sa));
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    fprintf(stderr, "usage: set_value VALUE [--host HOST] [--port PORT]\n");
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

  parse_args(argc, argv, &dst_host, &dst_port);

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
  set_valid_msg_magic(&msg);

  msg.led1_value = value;
  msg.led2_value = value;
  msg.led3_value = value;
  msg.led4_value = value;

  send_msg(sock, &sa, &msg);
}
