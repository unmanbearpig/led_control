
const uint16_t led_values_message_magic = 0x1324;

typedef struct __attribute__((__packed__)) {
  uint16_t magic;
  uint16_t led1_value;
  uint16_t led2_value;
  uint16_t led3_value;
  uint16_t led4_value;
} LedValuesMessage;

int is_msg_valid(LedValuesMessage *msg) {
  return msg->magic == led_values_message_magic;
}

void set_valid_msg_magic(LedValuesMessage *msg) {
  msg->magic = led_values_message_magic;
}

void set_msg_to_error_state(LedValuesMessage *msg) {
  set_valid_msg_magic(msg);
  msg->led1_value = 0;
  msg->led2_value = 0xFFFF;
  msg->led3_value = 0;
  msg->led4_value = 0xFFFF;
}
