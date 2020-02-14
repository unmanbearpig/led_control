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
  double led1;
  double led2;
  double led3;
  double led4;
} FLedState;

double stick_value(uint8_t x, uint8_t y) {
  int8_t sx = gamepad_abs_to_rel_axis(x);
  int8_t sy = gamepad_abs_to_rel_axis(y);

  return (sx / 127.0) * 0.000002 - pow((sy / 127.0), 3) * 0.01;
}

void print_state(FLedState *state) {
  if (verbose) {
    printf("%f %f %f %f\n", state->led1, state->led2, state->led3, state->led4);
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

void modify_msg_by_gamepad(FLedState *state, LedValuesMessage *msg, GamepadState *gamepad) {
  int8_t left_x, left_y, right_x, right_y = 0;

  double left_stick = stick_value(gamepad->left_x, gamepad->left_y);

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
    add_stick_value(&state->led1, left_stick);
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
    add_stick_value(&state->led2, left_stick);
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
    add_stick_value(&state->led3, left_stick);
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
    add_stick_value(&state->led4, left_stick);
  }

  print_state(state);

  if (gamepad->select_start_joystick_buttons_and_shoulders & START_BUTTON) {
    memset(state, 0, sizeof(*state));
  }

  msg->led1_value = state->led1 * 0xFFFF;
  msg->led2_value = state->led2 * 0xFFFF;
  msg->led3_value = state->led3 * 0xFFFF;
  msg->led4_value = state->led4 * 0xFFFF;

  if (gamepad->thumbs & RIGHT_THUMB_UP) {
    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
      msg->led1_value = 0xFFFF;
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
      msg->led2_value = 0xFFFF;
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
      msg->led3_value = 0xFFFF;
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
      msg->led4_value = 0xFFFF;
    }
  }
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
     { .name = "verbose", .has_arg = no_argument, .flag = &argid, .val = 'v' },
     { 0, 0, 0, 0 }
    };

  int longindex = 0;
  int ch = 0;

  while( (ch = getopt_long(argc, argv, "s:g:v", opts, &longindex)) != -1 ) {
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
    report_error(errno);
    return 1;
  }

  GamepadState gamepad = {};

  FLedState state;
  memset(&state, 0, sizeof(state));

  for(;;) {
    if (verbose) {
      printf("\e[1;1H\e[2J"); // clear screen
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
       .magic = led_values_message_magic,
       .led1_value = 0,
       .led2_value = 0,
       .led3_value = 0,
       .led4_value = 0,
      };

    modify_msg_by_gamepad(&state, &msg, &gamepad);

    if (verbose) {
      print_gamepad(&gamepad);
    }

    xfer_msg(spi_fd, &msg, verbose);
    fflush(stdout);
  }
}
