#include "protocol.h"
#include <stdio.h>

void print_msg(LedValuesMessage *msg) {
  printf(">    %04x %04x %04x %04x %04x\n", msg->magic, msg->led1_value, msg->led2_value, msg->led3_value, msg->led4_value);
}

void print_msg_bytes(uint16_t *buf) {
  const size_t len = sizeof(LedValuesMessage);
  printf(">    %lu:  %04x %04x %04x %04x %04x\n", len, ((uint16_t *)buf)[0], ((uint16_t *)buf)[1], ((uint16_t *)buf)[2], ((uint16_t *)buf)[3], ((uint16_t *)buf)[4]);
}
