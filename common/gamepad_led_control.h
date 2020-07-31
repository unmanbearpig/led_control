#include <string.h>
#include <math.h>
#include <stdio.h>
#include <sys/time.h>
#include "protocol.h"
#include "linux_gamepad.h"
#include "config.h"

typedef struct {
  double sine_amplitude;
  double sine_freq;
  double sine_freq_delta;
  double sine_phi;
  uint64_t last_upd;
} LedAttrs;

typedef struct {
  double value;
  double sine_target;
  LedAttrs attrs;
} Led;

typedef struct {
  GamepadState gamepad;
  int btn_map[4];
  float gamma;
  uint16_t pwm_period;
  double value_stick_sensitivity;
  double pwm_period_stick_sensitivity;
  double gamma_stick_sensitivity;
} GamepadLedControlState;

enum GamepadBtn {
  BTN_LU = 0,
  BTN_LD = 1,
  BTN_RU = 2,
  BTN_RD = 3,
};

void init_gamepad_led_control_state(GamepadLedControlState *gamepad_led_control) {
  memset(gamepad_led_control, 0, sizeof(GamepadLedControlState));

  gamepad_led_control->value_stick_sensitivity = 0.005;
  gamepad_led_control->gamma_stick_sensitivity = 0.002;
  gamepad_led_control->pwm_period_stick_sensitivity = 100.0;

  gamepad_led_control->gamma = 2.5;
  gamepad_led_control->pwm_period = PWM_PERIOD;

  gamepad_led_control->btn_map[BTN_LU] = 3;
  gamepad_led_control->btn_map[BTN_LD] = 0;
  gamepad_led_control->btn_map[BTN_RU] = 1;
  gamepad_led_control->btn_map[BTN_RD] = 2;
}

double stick_value(uint8_t x, uint8_t y) {
  int8_t sx = gamepad_abs_to_rel_axis(x);
  int8_t sy = gamepad_abs_to_rel_axis(y);

  // double value = ((sx / 127.0) * 0.000002) - (pow((sy / 127.0), 3) * 0.003);

  double value = -pow((sy / 127.0), 5.0) + pow((sx / 127.0), 5.0) * 0.02;

  // fprintf(stderr, "%f\n", value);

  return value;
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

void print_led(Led *led) {
  double df = led->attrs.sine_freq_delta;
  double f = led->attrs.sine_freq;

  fprintf(stderr, "val=%16f | df=%16f f=%16f\n", led->value, df, f);

  char bar[141] = {0};
  uint64_t bar_val = led->value * 140;
  if (bar_val > 140) {
    bar_val = 140;
  }

  uint64_t i = 0;
  for (i = 0; i < bar_val; i++) {
    bar[i] = '#';
  }
  bar[i+1] = 0;

  fprintf(stderr, "%s\n", bar);
}

void update_led_sine(Led *led, uint64_t t) {
  double new_val = led->sine_target;
  double f = led->attrs.sine_freq;
  double amp = led->attrs.sine_amplitude;
  double phi = led->attrs.sine_phi;

  uint64_t dt = t - led->attrs.last_upd;

  double delta = dt * f * 0.2;
  double new_sin = sin(phi);

  led->attrs.last_upd = t;

  add_stick_value(&new_val, (new_sin * amp) / 2.0);
  led->attrs.sine_phi += delta;

  led->value = new_val;
}

void update_led_sine_attrs(Led *led, double freq_delta, double amp_delta) {
  led->attrs.sine_freq += freq_delta;
  add_stick_value(&led->attrs.sine_amplitude, amp_delta);
}

uint64_t utime() {
  struct timeval tv;
  memset(&tv, 0, sizeof(tv));

  if (-1 == gettimeofday(&tv, NULL)) {
    fprintf(stderr, "gettimeofday error\n");
    return 0;
  }

  return tv.tv_sec * 1000000 + tv.tv_usec;
}

void update_leds_sine(Led *leds) {
  uint64_t t = utime();

  for (int i = 0; i < 4; i++) {
    update_led_sine(&leds[i], t);
  }
}

void handle_gamepad_led_config(LedValuesMessage *msg, GamepadLedControlState *control) {
  msg->type = LED_READ | LED_WRITE | LED_CONFIG;
  double left_stick = stick_value(control->gamepad.left_x, control->gamepad.left_y);

  if (control->gamepad.thumbs & RIGHT_THUMB_LEFT) {
    double delta = left_stick * control->gamma_stick_sensitivity;
    msg->payload.config.flags = LED_CONFIG_SET_GAMMA;

    control->gamma += delta;
    if (control->gamma < 0.0) {
      control->gamma = 0.0;
    }

    msg->payload.config.gamma = control->gamma;
    return;
  }

  if (control->gamepad.thumbs & RIGHT_THUMB_RIGHT) {
    msg->payload.config.flags = LED_CONFIG_SET_PWM_PERIOD;

    int16_t delta = (left_stick * control->pwm_period_stick_sensitivity);
    int32_t new_pwm_period = control->pwm_period + delta;
    if (new_pwm_period < 2) {
      new_pwm_period = 2;
    }

    if (new_pwm_period > 0xFFFF) {
      new_pwm_period = 0xFFFF;
    }

    control->pwm_period = new_pwm_period;
    // fprintf(stderr, "%d %i\n", control->pwm_period, delta);

    msg->payload.config.pwm_period = control->pwm_period;
    return;
  }
}

void handle_gamepad_led_values(Led *leds, LedValuesMessage *msg, GamepadLedControlState *control) {
  msg->type = LED_READ | LED_WRITE;
  msg->payload.data.flags = LED_VALUES_FLAG_FLOAT;
  msg->payload.data.amount = 1.0;

  double left_stick = stick_value(control->gamepad.left_x, control->gamepad.left_y) * control->value_stick_sensitivity;

  if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
    if (control->btn_map[BTN_LU] != -1) {
      add_stick_value(&leds[control->btn_map[BTN_LU]].sine_target, left_stick);
    }
  }

  if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
    if (control->btn_map[BTN_LD] != -1) {
      add_stick_value(&leds[control->btn_map[BTN_LD]].sine_target, left_stick);
    }
  }

  if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
    if (control->btn_map[BTN_RU] != -1) {
      add_stick_value(&leds[control->btn_map[BTN_RU]].sine_target, left_stick);
    }
  }

  if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
    if (control->btn_map[BTN_RD] != -1) {
      add_stick_value(&leds[control->btn_map[BTN_RD]].sine_target, left_stick);
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
    double freq_delta = stick_x_value(control->gamepad.right_x) * 0.000003;
    double amplitude_delta = stick_y_value(control->gamepad.right_y) * 0.007;

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
      if (control->btn_map[BTN_LU] != -1) {
        update_led_sine_attrs(&leds[control->btn_map[BTN_LU]], freq_delta, amplitude_delta);
      }
    }

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
      if (control->btn_map[BTN_LD] != -1) {
        update_led_sine_attrs(&leds[control->btn_map[BTN_LD]], freq_delta, amplitude_delta);
      }
    }

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
      if (control->btn_map[BTN_RU] != -1) {
        update_led_sine_attrs(&leds[control->btn_map[BTN_RU]], freq_delta, amplitude_delta);
      }
    }

    if (control->gamepad.select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
      if (control->btn_map[BTN_RD] != -1) {
        update_led_sine_attrs(&leds[control->btn_map[BTN_RD]], freq_delta, amplitude_delta);
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

  if (control->gamepad.thumbs & (RIGHT_THUMB_LEFT | RIGHT_THUMB_RIGHT)) {
    handle_gamepad_led_config(msg, control);
  } else {
    handle_gamepad_led_values(leds, msg, control);
  }
}
