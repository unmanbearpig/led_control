#include "protocol.h"
#include <stdio.h>

void print_msg_bytes(uint16_t *buf) {
  const size_t len = sizeof(LedValuesMessage);
  printf(">    %lu:  %04x %04x %04x %04x %04x\n", len, ((uint16_t *)buf)[0], ((uint16_t *)buf)[1], ((uint16_t *)buf)[2], ((uint16_t *)buf)[3], ((uint16_t *)buf)[4]);
}
