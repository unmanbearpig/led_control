#pragma once

#include <inttypes.h>

#define LED_VALUES_MESSAGE_MAGIC 0x1324
#define LED_VALUES_MESSAGE_READ_REQUEST_MAGIC 0xFEED;

typedef struct __attribute__((__packed__)) {
  uint16_t magic;
  uint16_t led1_value;
  uint16_t led2_value;
  uint16_t led3_value;
  uint16_t led4_value;
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
  msg->led1_value = 0;
  msg->led2_value = 0xFFFF;
  msg->led3_value = 0;
  msg->led4_value = 0xFFFF;
}

void set_all_msg_values(LedValuesMessage *msg, uint16_t value) {
  msg->led1_value = value;
  msg->led2_value = value;
  msg->led3_value = value;
  msg->led4_value = value;
}
