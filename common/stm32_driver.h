#include "protocol.h"
#include "stm32_pwm.h"

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

void led_values_convert_float_to_16(uint16_t pwm_period, float led_gamma, uint16_t *led_values16, float *led_values_float) {
  for(int i = 0; i < LED_COUNT; i++) {
    float val = powf(led_values_float[i], led_gamma) * pwm_period;
    if (val > pwm_period) {
      val = pwm_period;
    }

    led_values16[i] = val;
  }
}

void handle_values_msg(uint16_t *led_values, float *float_led_values, uint16_t pwm_period,
                       float *led_gamma,
                       LedValuesMessage *input_msg, LedValuesMessage *output_msg) {
  for(int i = 0; i < LED_COUNT; i++) {
    output_msg->payload.data.values.values16[i] = led_values[i];
  }

  output_msg->type = output_msg->type & ~LED_CONFIG;

  int use_float = false;

  if (input_msg->type & LED_WRITE) {
    if (input_msg->payload.data.flags & LED_VALUES_FLAG_FLOAT) {
      use_float = true;

      for(int i = 0; i < LED_COUNT; i++) {
        float input_value = input_msg->payload.data.values.values_float[i];
        float amount = input_msg->payload.data.amount;
        float old_value = float_led_values[i];

        float val = 0;
        if (input_msg->payload.data.flags & LED_VALUES_FLAG_ADD) {
          val = old_value + (amount * input_value);
        } else {
          val = (old_value * (1.0 - amount)) + (amount * input_value);
        }

        float_led_values[i] = val;
      }

      led_values_convert_float_to_16(pwm_period, *led_gamma, led_values, float_led_values);
      set_4_leds(led_values, pwm_period);
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
      memcpy(&output_msg->payload.data.values.values_float, float_led_values, sizeof(float_led_values));
    } else {
      output_msg->payload.data.flags = 0;
      memcpy(&output_msg->payload.data.values.values16, led_values, sizeof(led_values));
    }
  }
}

void handle_config_msg(uint16_t *pwm_period, float *led_gamma,
                       uint16_t *led_values, float *float_led_values,
                       LedValuesMessage *input_msg, LedValuesMessage *output_msg) {
  if (input_msg->type & LED_WRITE) {
    if (input_msg->payload.config.flags & LED_CONFIG_SET_GAMMA) {
      *led_gamma = input_msg->payload.config.gamma;
    }

    if (input_msg->payload.config.flags & LED_CONFIG_SET_PWM_PERIOD) {
      uint16_t new_pwm_period = input_msg->payload.config.pwm_period;
      if (new_pwm_period != *pwm_period) {
        *pwm_period = new_pwm_period;
        set_pwm_period(*pwm_period);
      }
    }

    led_values_convert_float_to_16(*pwm_period, *led_gamma, led_values, float_led_values);
    set_4_leds(led_values, *pwm_period);
  }

  if (input_msg->type & LED_READ) {
    memset(output_msg, 0, sizeof(*output_msg));
    output_msg->type = LED_CONFIG;
    output_msg->payload.config.gamma = *led_gamma;
    output_msg->payload.config.pwm_period = *pwm_period;
    // TODO
  }
}

void handle_msg(uint16_t *led_values, float *float_led_values, uint16_t *pwm_period,
                float *led_gamma,
                LedValuesMessage *input_msg, LedValuesMessage *output_msg,
                void (*on_connection_error)()) {
  if (is_all_zeros(input_msg, sizeof(*input_msg))) {
    output_msg->magic = LED_VALUES_MESSAGE_MAGIC;
    output_msg->type = LED_READ | LED_WRITE;
    output_msg->payload.data.flags = 0;
    output_msg->payload.data.values.values16[0] = led_values[0];
    output_msg->payload.data.values.values16[1] = led_values[1];
    output_msg->payload.data.values.values16[2] = led_values[2];
    output_msg->payload.data.values.values16[3] = led_values[3];
  } else if (is_msg_valid(input_msg)) {
    output_msg->magic = LED_VALUES_MESSAGE_MAGIC;
    if (input_msg->type & LED_CONFIG) {
      handle_config_msg(pwm_period, led_gamma, led_values, float_led_values, input_msg, output_msg);
    } else {
      handle_values_msg(led_values, float_led_values, *pwm_period, led_gamma, input_msg, output_msg);
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

    led_values[0] = 0;
    led_values[1] = *pwm_period;
    led_values[2] = 0;
    led_values[3] = *pwm_period;
  }
}
