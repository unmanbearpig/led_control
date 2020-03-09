#pragma once

#include <inttypes.h>

#define UDP_SLEEP_US 5000

#define LED_COUNT 4
#define LED_VALUES_MESSAGE_MAGIC 0x1324

#define LED_READ 1
#define LED_WRITE 2
#define LED_CONFIG 4 // otherwise values

#define LED_CONFIG_SET_GAMMA 1
#define LED_CONFIG_SET_PWM_PERIOD 2

typedef struct __attribute__((__packed__)) {
  uint16_t flags;
  float gamma;
  uint16_t pwm_period;
} LedMsgConfig;

#define LED_VALUES_FLAG_FLOAT 1 // otherwise uint16_t
#define LED_VALUES_FLAG_ADD 2 // otherwise replace

union LedValues {
  uint16_t values16[LED_COUNT];
  float values_float[LED_COUNT];
};

typedef struct __attribute__((__packed__)) {
  uint16_t flags;
  float amount;
  union LedValues values;
} LedMsgData;

union LedMsgPayload {
  LedMsgData data;
  LedMsgConfig config;
};

typedef struct __attribute__((__packed__)) {
  uint16_t magic;
  uint16_t type;
  union LedMsgPayload payload;
  uint16_t blah1;
  uint32_t blah2;
} LedValuesMessage;

int is_msg_valid(LedValuesMessage *msg) {
  return msg->magic == LED_VALUES_MESSAGE_MAGIC;
}

void set_valid_msg_magic(LedValuesMessage *msg) {
  msg->magic = LED_VALUES_MESSAGE_MAGIC;
}

void set_msg_to_error_state(LedValuesMessage *msg) {
  set_valid_msg_magic(msg);
  msg->type = LED_WRITE | LED_READ;
  msg->payload.data.flags = 0;
  msg->payload.data.values.values16[0] = 0;
  msg->payload.data.values.values16[1] = 0xFFFF;
  msg->payload.data.values.values16[2] = 0;
  msg->payload.data.values.values16[3] = 0xFFFF;
}

void set_all_msg_values(LedValuesMessage *msg, uint16_t value) {
  msg->payload.data.values.values16[0] = value;
  msg->payload.data.values.values16[1] = value;
  msg->payload.data.values.values16[2] = value;
  msg->payload.data.values.values16[3] = value;
}

void set_all_msg_float_values(LedValuesMessage *msg, float value) {
  msg->payload.data.values.values_float[0] = value;
  msg->payload.data.values.values_float[1] = value;
  msg->payload.data.values.values_float[2] = value;
  msg->payload.data.values.values_float[3] = value;
}
