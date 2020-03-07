#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/types.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <math.h>
#include <unistd.h>
#include <getopt.h>
#include <sys/time.h>
#include "../common/linux_gamepad.h"
#include "../common/linux_gamepad_print.h"
#include "../common/linux_spi.h"
#include "../common/protocol.h"
#include "../common/linux_util.h"
#include "../common/linux_spi_protocol.h"

int verbose = 0;

void report_error(int err) {
  char *error_str = strerror(err);
  fprintf(stderr, "Error: %s\n", error_str);
}

typedef struct {
  double sine_amplitude;
  double sine_freq;
} LedAttrs;

typedef struct {
  double value;
  LedAttrs attrs;
} Led;

#define BTN_LU 0
#define BTN_LD 1
#define BTN_RU 2
#define BTN_RD 3

unsigned int btn_map[] = { 0, 1, 2, 3 };

double stick_value(uint8_t x, uint8_t y) {
  int8_t sx = gamepad_abs_to_rel_axis(x);
  int8_t sy = gamepad_abs_to_rel_axis(y);

  return (sx / 127.0) * 0.000002 - pow((sy / 127.0), 3) * 0.01;
}

double stick_x_value(uint8_t val) {
  return pow((gamepad_abs_to_rel_axis(val) / 127.0), 3) * 0.01;
}

double stick_y_value(uint8_t val) {
  return -stick_x_value(val);
}

void print_state(Led *leds) {
  if (verbose) {
    fprintf(stderr, "%f %f %f %f\n%e %e %e %e\n%e %e %e %e\n",
            leds[0].value, leds[1].value, leds[2].value, leds[3].value,
            leds[0].attrs.sine_amplitude, leds[1].attrs.sine_amplitude,
            leds[2].attrs.sine_amplitude, leds[3].attrs.sine_amplitude,
            leds[0].attrs.sine_freq, leds[1].attrs.sine_freq,
            leds[2].attrs.sine_freq, leds[3].attrs.sine_freq
            );
  }
}

void add_stick_value(double *to, double from) {
  *to += from;
  if (*to < 0.0) {
    *to = 0.0;
  } else if (*to > 1.0) {
    *to = 1.0;
  }
}

#define DEFAULT_ADJUSTMENT_EXP 7.0
double adjustment_exp = DEFAULT_ADJUSTMENT_EXP;

double adjust_led_value(double value) {
  return pow(value, adjustment_exp);
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

void modify_msg_by_gamepad(Led *leds, LedValuesMessage *msg, GamepadState *gamepad) {
  int8_t left_x, left_y, right_x, right_y = 0;

  double left_stick = stick_value(gamepad->left_x, gamepad->left_y);
  double right_stick = stick_value(gamepad->right_x, gamepad->right_y);

  if (gamepad->select_start_joystick_buttons_and_shoulders & RIGHT_JOYSTICK_BUTTON) {
    adjustment_exp += right_stick;
  }

  if (verbose) {
    fprintf(stderr, "adjustment_exp: %f\n", adjustment_exp);
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
    add_stick_value(&leds[btn_map[BTN_LU]].value, left_stick);
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
    add_stick_value(&leds[btn_map[BTN_LD]].value, left_stick);
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
    add_stick_value(&leds[btn_map[BTN_RU]].value, left_stick);
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
    add_stick_value(&leds[btn_map[BTN_RD]].value, left_stick);
  }

  print_state(leds);

  if (gamepad->select_start_joystick_buttons_and_shoulders & START_BUTTON) {
    memset(leds, 0, sizeof(Led) * 4);
    adjustment_exp = DEFAULT_ADJUSTMENT_EXP;
  }

  uint16_t converted_values[4] =
    {
     adjust_led_value(leds[0].value) * 0xFFFF,
     adjust_led_value(leds[1].value) * 0xFFFF,
     adjust_led_value(leds[2].value) * 0xFFFF,
     adjust_led_value(leds[3].value) * 0xFFFF,
    };

  if (gamepad->thumbs & RIGHT_THUMB_UP) {
    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
      converted_values[btn_map[BTN_LU]] = 0xFFFF;
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
      converted_values[btn_map[BTN_LD]] = 0xFFFF;
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
      converted_values[btn_map[BTN_RU]] = 0xFFFF;
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
      converted_values[btn_map[BTN_RD]] = 0xFFFF;
    }
  } else {
    double freq_delta = stick_x_value(gamepad->right_x) * 0.00001;
    double amplitude_delta = stick_y_value(gamepad->right_y) * 0.01;

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
      leds[btn_map[BTN_LU]].attrs.sine_freq += freq_delta;
      add_stick_value(&leds[btn_map[BTN_LU]].attrs.sine_amplitude, amplitude_delta);
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
      leds[btn_map[BTN_LD]].attrs.sine_freq += freq_delta;
      add_stick_value(&leds[btn_map[BTN_LD]].attrs.sine_amplitude, amplitude_delta);
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
      leds[btn_map[BTN_RU]].attrs.sine_freq += freq_delta;
      add_stick_value(&leds[btn_map[BTN_RU]].attrs.sine_amplitude, amplitude_delta);
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
      leds[btn_map[BTN_RD]].attrs.sine_freq += freq_delta;
      add_stick_value(&leds[btn_map[BTN_RD]].attrs.sine_amplitude, amplitude_delta);
    }

    for (int i = 0; i < 4; i++) {
      if (leds[i].attrs.sine_freq < 0.0) {
        leds[i].attrs.sine_freq = 0.0;
      }
    }
  }

  msg->led_values[0] = converted_values[0];
  msg->led_values[1] = converted_values[1];
  msg->led_values[2] = converted_values[2];
  msg->led_values[3] = converted_values[3];
}

void print_usage() {
  fprintf(stderr, "--spi <path to spi dev (default=/dev/spidev0.0)>\n--gamepad <path to gamepad hidraw, default=/dev/hidraw0>\n--verbose - enable verbose output\n");
}

int parse_args(int argc, char *argv[], char **spi_path, char **gamepad_path) {
  int argid = 0;

  struct option opts[] =
    {
     { .name = "spi", .has_arg = required_argument, .flag = &argid, .val = 's' },
     { .name = "gamepad", .has_arg = required_argument, .flag = &argid, .val = 'g' },
     { .name = "btn-lu", .has_arg = required_argument, .flag = &argid, .val = 'c' },
     { .name = "btn-ld", .has_arg = required_argument, .flag = &argid, .val = 't' },
     { .name = "btn-ru", .has_arg = required_argument, .flag = &argid, .val = 'r' },
     { .name = "btn-rd", .has_arg = required_argument, .flag = &argid, .val = 'n' },
     { .name = "verbose", .has_arg = no_argument, .flag = &argid, .val = 'v' },
     { 0, 0, 0, 0 }
    };

  int longindex = 0;
  int ch = 0;

  while( (ch = getopt_long(argc, argv, "s:g:c:t:r:n:v", opts, &longindex)) != -1 ) {
    switch(argid) {
    case 's':
      *spi_path = optarg;
      break;
    case 'g':
      *gamepad_path = optarg;
      break;
    case 'v':
      verbose = 1;
      break;
    case 'c':
      btn_map[BTN_LU] = atoi(optarg);
      break;
    case 't':
      btn_map[BTN_LD] = atoi(optarg);
      break;
    case 'r':
      btn_map[BTN_RU] = atoi(optarg);
      break;
    case 'n':
      btn_map[BTN_RD] = atoi(optarg);
      break;
    default:
      fprintf(stderr, "Invalid argument\n");
      print_usage();
      return(0);
      break;
    }
  }

  return(1);
}

int main(int argc, char *argv[]) {
  char *spi_path = "/dev/spidev0.0";
  char *gamepad_path = DEFAULT_GAMEPAD_PATH;

  if (!parse_args(argc, argv, &spi_path, &gamepad_path)) {
    exit(1);
  }

  if (verbose) {
    char buf[4096];
    memset(buf, 0, sizeof(buf));
    setbuffer(stderr, buf, sizeof(buf));
  }

  int spi_fd = 0;
  if (0 == strcmp(spi_path, "-")) {
    spi_fd = STDOUT_FILENO;
  } else {
    spi_fd = try_open_spi(spi_path, 0);
  }

  int gamepad_fd = 0;

  if (0 == strcmp(gamepad_path, "-")) {
    gamepad_fd = STDIN_FILENO;
  } else {
    gamepad_fd = open(gamepad_path, O_RDONLY);
  }

  if (gamepad_fd == -1) {
    fprintf(stderr, "Gamepad: ");
    report_error(errno);
    return 1;
  }

  GamepadState gamepad = {};

  Led leds[4];
  memset(leds, 0, sizeof(leds));

  for(;;) {
    if (verbose) {
      fprintf(stderr, "\e[1;1H\e[2J"); // clear screen
    }

    ssize_t bytes_read = read(gamepad_fd, &gamepad, sizeof(gamepad));

    if (bytes_read != sizeof(gamepad)) {
      int err = errno;
      fprintf(stderr, "%ld bytes read, this is an error\n", bytes_read);
      report_error(err);
      return 1;
    }

    LedValuesMessage msg =
      {
       .magic = LED_VALUES_MESSAGE_MAGIC,
       .led_values = { 0, 0, 0, 0 }
      };

    update_leds_sine(leds);
    modify_msg_by_gamepad(leds, &msg, &gamepad);

    if (verbose) {
      print_gamepad(&gamepad);
    }

    if (spi_fd == STDOUT_FILENO) {
      write(spi_fd, &msg, sizeof(msg));
      fflush(stdout);
    } else {
      xfer_msg(spi_fd, &msg, verbose);
    }

    if (verbose) {
      fflush(stderr);
    }
  }
}
