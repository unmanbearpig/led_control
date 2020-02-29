#include <inttypes.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include "../common/protocol.h"
#include "../common/protocol_debug.h"
#include <math.h>

int send_msg(int sock, struct sockaddr_in *sa, LedValuesMessage *msg) {
  return sendto(sock, msg, sizeof(*msg), 0, (struct sockaddr *)sa, sizeof(*sa));
  // printf("%d\n", msg->led1_value);
}

void send_sine(int sock, struct sockaddr_in *sa) {
  uint64_t i = 0;
  LedValuesMessage msg = { 0 };
  set_valid_msg_magic(&msg);

  int sleep_us = 5000;

  double speed = 0.01;

  while(1) {
    double s = (sin((i * speed)) + 1) / 2.0;

    s = pow(s, 2.0);

    uint16_t min = 5000;
    uint16_t max = 40000;

    uint16_t v = min + (s * (max - min));

    set_all_msg_values(&msg, v);
    send_msg(sock, sa, &msg);
    usleep(sleep_us);
    i++;
  }
}

int main(int argc, char *argv[]) {
  int sock = 0;

  char *dst_host = "192.168.0.102";
  int dst_port = 8932;

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

  send_sine(sock, &sa);


  LedValuesMessage msg;
  memset(&msg, 0, sizeof(msg));
  set_valid_msg_magic(&msg);

  int sleep_us = 5000;

  uint16_t min = 16;
  uint16_t max = 230;

  for(uint16_t i = min; i < max; i++) {
    uint16_t j = i * i;
    usleep(sleep_us);

    msg.led1_value = j;
    msg.led2_value = j;
    msg.led3_value = j;
    msg.led4_value = j;

    int bytes_sent = send_msg(sock, &sa, &msg);
    // printf("bytes_sent = %d\n", bytes_sent);
  }

  for(uint16_t i = max; i > min; i--) {
    uint16_t j = i * i;
    usleep(sleep_us);

    msg.led1_value = j;
    msg.led2_value = j;
    msg.led3_value = j;
    msg.led4_value = j;

    int bytes_sent = send_msg(sock, &sa, &msg);
    // printf("bytes_sent = %d\n", bytes_sent);
  }
}
