#include "protocol.h"
#include "stm32_pwm.h"

struct driver_state {
  uint16_t led_values[LED_COUNT + 1];
  float float_led_values[LED_COUNT];
  uint16_t pwm_period;
  int use_float;
  float led_gamma;
  int num_leds;
  enum tim_oc_id oc_channels[4];
};

void init_driver_state(struct driver_state *state, uint16_t initial_led_value, int num_leds,
                       enum tim_oc_id *oc_channels) {
  state->pwm_period = 16383;
  state->use_float = 0;
  state->led_gamma = 2.0;
  state->num_leds = num_leds;
  state->led_values[0] = initial_led_value;
  state->led_values[1] = initial_led_value;
  state->led_values[2] = initial_led_value;
  state->led_values[3] = initial_led_value;
  memcpy(state->oc_channels, oc_channels, sizeof(oc_channels[0]) * num_leds);
}

void set_leds(struct driver_state *state) {
  for (int i = 0; i < state->num_leds; i++) {
    timer_set_oc_value(TIM1, state->oc_channels[i], (state->pwm_period - state->led_values[i]));
  }
}

// not optimized at all
int is_all_zeros(void *buf, size_t len) {
  uint8_t *chars = (uint8_t *)buf;
  for (size_t i = 0; i < len; i++) {
    if (chars[i] != 0) {
      return 0;
    }
  }

  return 1;
}

void led_values_convert_float_to_16(struct driver_state *state) {
  for(int i = 0; i < state->num_leds; i++) {
    float val = powf(state->float_led_values[i], state->led_gamma) * state->pwm_period;
    if (val > state->pwm_period) {
      val = state->pwm_period;
    }

    state->led_values[i] = val;
  }
}

void handle_values_msg(struct driver_state *state,
                       LedValuesMessage *input_msg, LedValuesMessage *output_msg) {
  for(int i = 0; i < LED_COUNT; i++) {
    output_msg->payload.data.values.values16[i] = state->led_values[i];
  }

  output_msg->type = output_msg->type & ~LED_CONFIG;

  int use_float = false;

  if (input_msg->type & LED_WRITE) {
    if (input_msg->payload.data.flags & LED_VALUES_FLAG_FLOAT) {
      use_float = true;

      for(int i = 0; i < state->num_leds; i++) {
        float input_value = input_msg->payload.data.values.values_float[i];
        float amount = input_msg->payload.data.amount;
        float old_value = state->float_led_values[i];

        float val = 0;
        if (input_msg->payload.data.flags & LED_VALUES_FLAG_ADD) {
          val = old_value + (amount * input_value);
        } else {
          val = (old_value * (1.0 - amount)) + (amount * input_value);
        }

        state->float_led_values[i] = val;
      }

      led_values_convert_float_to_16(state);
      set_leds(state);
    } else {
      // if (input_msg->type & LED_WRITE) {
      //   use_float = false;

      //   for(int i = 0; i < LED_COUNT; i++) {
      //     led_values[i] = input_msg->payload.data.values.values16[i];
      //   }

      //   set_4_leds(led_values);
      // }

      // output_msg->magic = LED_VALUES_MESSAGE_MAGIC;
      // output_msg->payload.data.values.values16[0] = led_values[0];
      // output_msg->payload.data.values.values16[1] = led_values[1];
      // output_msg->payload.data.values.values16[2] = led_values[2];
      // output_msg->payload.data.values.values16[3] = led_values[3];
    }
  }

  if (input_msg->type & LED_READ) {
    if (input_msg->payload.data.flags & LED_VALUES_FLAG_FLOAT) {
      output_msg->payload.data.flags = LED_VALUES_FLAG_FLOAT;
      memcpy(&output_msg->payload.data.values.values_float,
             state->float_led_values, sizeof(state->float_led_values));
    } else {
      output_msg->payload.data.flags = 0;
      memcpy(&output_msg->payload.data.values.values16,
             state->led_values, sizeof(state->led_values));
    }
  }
}

void handle_config_msg(struct driver_state *state,
                       LedValuesMessage *input_msg, LedValuesMessage *output_msg) {
  if (input_msg->type & LED_WRITE) {
    if (input_msg->payload.config.flags & LED_CONFIG_SET_GAMMA) {
      state->led_gamma = input_msg->payload.config.gamma;
    }

    if (input_msg->payload.config.flags & LED_CONFIG_SET_PWM_PERIOD) {
      uint16_t new_pwm_period = input_msg->payload.config.pwm_period;
      if (new_pwm_period != state->pwm_period) {
        state->pwm_period = new_pwm_period;
        set_pwm_period(state->pwm_period);
      }
    }

    led_values_convert_float_to_16(state);
    set_leds(state);
  }

  if (input_msg->type & LED_READ) {
    memset(output_msg, 0, sizeof(*output_msg));
    output_msg->type = LED_CONFIG;
    output_msg->payload.config.gamma = state->led_gamma;
    output_msg->payload.config.pwm_period = state->pwm_period;
    // TODO
  }
}

void handle_msg(struct driver_state *state,
                LedValuesMessage *input_msg, LedValuesMessage *output_msg,
                void (*on_connection_error)()) {
  if (is_all_zeros(input_msg, sizeof(*input_msg))) {
    output_msg->magic = LED_VALUES_MESSAGE_MAGIC;
    output_msg->type = LED_READ | LED_WRITE;
    output_msg->payload.data.flags = 0;
    output_msg->payload.data.values.values16[0] = state->led_values[0];
    output_msg->payload.data.values.values16[1] = state->led_values[1];
    output_msg->payload.data.values.values16[2] = state->led_values[2];
    output_msg->payload.data.values.values16[3] = state->led_values[3];
  } else if (is_msg_valid(input_msg)) {
    output_msg->magic = LED_VALUES_MESSAGE_MAGIC;
    if (input_msg->type & LED_CONFIG) {
      handle_config_msg(state, input_msg, output_msg);
    } else {
      handle_values_msg(state, input_msg, output_msg);
    }
  } else {
    // We (the slave) might go out of sync with the host,
    // i.e. we incorrectly assume the start of the message
    // it might happen if the slave was (re)started in the middle of communication
    // Here we try to resynchronize with the host by just restarting SPI DMA
    // Eventually we should catch the correct start of the message and stop receiving errors
    // There probably is a better way, but it works for now
    (*on_connection_error)();
    // Debugging, so I notice the errors better.
    // Should change it no not changing state

    state->led_values[0] = 0;
    state->led_values[1] = state->pwm_period;
    state->led_values[2] = 0;
    state->led_values[3] = state->pwm_period;
  }
}
