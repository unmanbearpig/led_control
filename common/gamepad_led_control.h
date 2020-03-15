#include <string.h>
#include <math.h>
#include <stdio.h>
#include <sys/time.h>
#include "protocol.h"
#include "linux_gamepad.h"

typedef struct {
  double sine_amplitude;
  double sine_freq;
} LedAttrs;

typedef struct {
  double value;
  LedAttrs attrs;
} Led;

typedef struct {
  GamepadState gamepad;
  int btn_map[4];
  float gamma;
} GamepadLedControlState;

enum GamepadBtn {
                 BTN_LU = 0,
                 BTN_LD = 1,
                 BTN_RU = 2,
                 BTN_RD = 3,
};

void init_gamepad_led_control_state(GamepadLedControlState *gamepad_led_control) {
  memset(gamepad_led_control, 0, sizeof(GamepadLedControlState));

  gamepad_led_control->gamma = 2.0;

  gamepad_led_control->btn_map[BTN_LU] = 3;
  gamepad_led_control->btn_map[BTN_LD] = 0;
  gamepad_led_control->btn_map[BTN_RU] = 1;
  gamepad_led_control->btn_map[BTN_RD] = 2;
}

double stick_value(uint8_t x, uint8_t y) {
  int8_t sx = gamepad_abs_to_rel_axis(x);
  int8_t sy = gamepad_abs_to_rel_axis(y);

  return (sx / 127.0) * 0.000002 - pow((sy / 127.0), 3) * 0.003;
}

double stick_x_value(uint8_t val) {
  return pow((gamepad_abs_to_rel_axis(val) / 127.0), 3) * 0.01;
}

double stick_y_value(uint8_t val) {
  return -stick_x_value(val);
}

void add_stick_value(double *to, double from) {
  *to += from;
  if (*to < 0.0) {
    *to = 0.0;
  } else if (*to > 1.0) {
    *to = 1.0;
  }
}

void update_led_sine(Led *led, uint64_t t) {
  add_stick_value(&led->value, sin(t * led->attrs.sine_freq) * led->attrs.sine_amplitude / 2.0);
}

void update_leds_sine(Led *leds) {
  struct timeval tv;
  memset(&tv, 0, sizeof(tv));

  if (-1 == gettimeofday(&tv, NULL)) {
    fprintf(stderr, "gettimeofday error\n");
    return;
  }

  uint64_t t = tv.tv_sec * 1000000 + tv.tv_usec;

  for (int i = 0; i < 4; i++) {
    update_led_sine(&leds[i], t);
  }
}

void handle_gamepad_led_config(LedValuesMessage *msg, GamepadLedControlState *control) {
  msg->type = LED_READ | LED_WRITE | LED_CONFIG;
  msg->payload.config.flags = LED_CONFIG_SET_GAMMA;

  double left_stick = stick_value(control->gamepad.left_x, control->gamepad.left_y);

  control->gamma += left_stick;
  if (control->gamma < 0.0) {
    control->gamma = 0.0;
  }

  msg->payload.config.gamma = control->gamma;

}

void handle_gamepad_led_values(Led *leds, LedValuesMessage *msg, GamepadLedControlState *control) {
  msg->type = LED_READ | LED_WRITE;
  msg->payload.data.flags = LED_VALUES_FLAG_FLOAT;
  msg->payload.data.amount = 1.0;

  double left_stick = stick_value(control->gamepad.left_x, control->gamepad.left_y);

  if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
    if (control->btn_map[BTN_LU] != -1) {
      add_stick_value(&leds[control->btn_map[BTN_LU]].value, left_stick);
    }
  }

  if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
    if (control->btn_map[BTN_LD] != -1) {
      add_stick_value(&leds[control->btn_map[BTN_LD]].value, left_stick);
    }
  }

  if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
    if (control->btn_map[BTN_RU] != -1) {
      add_stick_value(&leds[control->btn_map[BTN_RU]].value, left_stick);
    }
  }

  if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
    if (control->btn_map[BTN_RD] != -1) {
      add_stick_value(&leds[control->btn_map[BTN_RD]].value, left_stick);
    }
  }

  if (control->gamepad.select_start_joystick_buttons_and_shoulders & START_BUTTON) {
    memset(leds, 0, sizeof(Led) * 4);
  }

  float values[LED_COUNT] = { 0, 0, 0, 0 };

  for(int i = 0; i < LED_COUNT; i++) {
    values[i] = leds[i].value;
  }

  if (control->gamepad.thumbs & RIGHT_THUMB_UP) {
    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
      if (control->btn_map[BTN_LU] != -1) {
        values[control->btn_map[BTN_LU]] = 1.0;
      }
    }

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
      if (control->btn_map[BTN_LD] != -1) {
        values[control->btn_map[BTN_LD]] = 1.0;
      }
    }

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
      if (control->btn_map[BTN_RU] != -1) {
        values[control->btn_map[BTN_RU]] = 1.0;
      }
    }

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
      if (control->btn_map[BTN_RD] != -1) {
        values[control->btn_map[BTN_RD]] = 1.0;
      }
    }
  } else {
    double freq_delta = stick_x_value(control->gamepad.right_x) * 0.00001;
    double amplitude_delta = stick_y_value(control->gamepad.right_y) * 0.01;

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
      if (control->btn_map[BTN_LU] != -1) {
        leds[control->btn_map[BTN_LU]].attrs.sine_freq += freq_delta;
        add_stick_value(&leds[control->btn_map[BTN_LU]].attrs.sine_amplitude, amplitude_delta);
      }
    }

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
      if (control->btn_map[BTN_LD] != -1) {
        leds[control->btn_map[BTN_LD]].attrs.sine_freq += freq_delta;
        add_stick_value(&leds[control->btn_map[BTN_LD]].attrs.sine_amplitude, amplitude_delta);
      }
    }

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
      if (control->btn_map[BTN_RU] != -1) {
        leds[control->btn_map[BTN_RU]].attrs.sine_freq += freq_delta;
        add_stick_value(&leds[control->btn_map[BTN_RU]].attrs.sine_amplitude, amplitude_delta);
      }
    }

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
      if (control->btn_map[BTN_RD] != -1) {
        leds[control->btn_map[BTN_RD]].attrs.sine_freq += freq_delta;
        add_stick_value(&leds[control->btn_map[BTN_RD]].attrs.sine_amplitude, amplitude_delta);
      }
    }

    for (int i = 0; i < 4; i++) {
      if (leds[i].attrs.sine_freq < 0.0) {
        leds[i].attrs.sine_freq = 0.0;
      }
    }
  }

  msg->payload.data.values.values_float[0] = values[0];
  msg->payload.data.values.values_float[1] = values[1];
  msg->payload.data.values.values_float[2] = values[2];
  msg->payload.data.values.values_float[3] = values[3];
}

void modify_msg_by_gamepad(Led *leds, LedValuesMessage *msg, GamepadLedControlState *control) {
  /* int8_t left_x, left_y, right_x, right_y = 0; */

  if (control->gamepad.thumbs & RIGHT_THUMB_LEFT) {
    handle_gamepad_led_config(msg, control);
  } else {
    handle_gamepad_led_values(leds, msg, control);
  }
}
