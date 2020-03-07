#include "protocol.h"
#include <stdio.h>

void print_msg(LedValuesMessage *msg, char *label) {
  if (!label) {
    label = ">";
  }

  printf("%s    %04x %04x %04x %04x %04x\n", label, msg->magic, msg->led_values[0], msg->led_values[1], msg->led_values[2], msg->led_values[3]);
}

void print_msg_bytes(uint16_t *buf) {
  const size_t len = sizeof(LedValuesMessage);
  printf(">    %lu:  %04x %04x %04x %04x %04x\n", len, ((uint16_t *)buf)[0], ((uint16_t *)buf)[1], ((uint16_t *)buf)[2], ((uint16_t *)buf)[3], ((uint16_t *)buf)[4]);
}
