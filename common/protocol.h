#pragma once

#include <inttypes.h>

#define LED_VALUES_MESSAGE_MAGIC 0x1324
#define LED_VALUES_MESSAGE_READ_REQUEST_MAGIC 0xFEED;

#define LED_READ 1
#define LED_WRITE 2
#define LED_VALUES 4
#define LED_CONFIG 8

typedef struct __attribute__((__packed__)) {
  uint16_t magic;
  uint16_t type;
  uint16_t led_values[4];
} LedValuesMessage;

int is_msg_valid(LedValuesMessage *msg) {
  return msg->magic == LED_VALUES_MESSAGE_MAGIC;
}

int is_msg_read_request(LedValuesMessage *msg) {
  return msg->magic == LED_VALUES_MESSAGE_READ_REQUEST_MAGIC;
}

void set_valid_msg_magic(LedValuesMessage *msg) {
  msg->magic = LED_VALUES_MESSAGE_MAGIC;
}

void set_msg_to_error_state(LedValuesMessage *msg) {
  set_valid_msg_magic(msg);
  msg->led_values[0] = 0;
  msg->led_values[1] = 0xFFFF;
  msg->led_values[2] = 0;
  msg->led_values[3] = 0xFFFF;
}

void set_all_msg_values(LedValuesMessage *msg, uint16_t value) {
  msg->led_values[0] = value;
  msg->led_values[1] = value;
  msg->led_values[2] = value;
  msg->led_values[3] = value;
}
