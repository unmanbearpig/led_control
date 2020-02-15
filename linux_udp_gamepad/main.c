#include <inttypes.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include "../common/protocol.h"
#include "../common/protocol_debug.h"

int send_msg(int sock, struct sockaddr_in *sa, LedValuesMessage *msg) {
  return sendto(sock, msg, sizeof(*msg), 0, (struct sockaddr *)sa, sizeof(*sa));
}

int main(int argc, char *argv[]) {
  int sock = 0;

  char *dst_host = "192.168.0.105";
  int dst_port = 8921;

  sock = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
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

  int sleep_us = 10;

  for(uint16_t i = 0; i < 65535; i++) {
    usleep(sleep_us);

    msg.led1_value = i;
    msg.led2_value = i;
    msg.led3_value = i;
    msg.led4_value = i;

    int bytes_sent = send_msg(sock, &sa, &msg);
    // printf("bytes_sent = %d\n", bytes_sent);
  }

  for(uint16_t i = 65535; i > 0; i--) {
    usleep(sleep_us);

    msg.led1_value = i;
    msg.led2_value = i;
    msg.led3_value = i;
    msg.led4_value = i;

    int bytes_sent = send_msg(sock, &sa, &msg);
    // printf("bytes_sent = %d\n", bytes_sent);
  }
}
