#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/types.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <math.h>
#include "../common/linux_gamepad.h"
#include "../common/linux_gamepad_print.h"
#include "../common/linux_spi.h"
#include "../common/protocol.h"
#include "../common/linux_util.h"
#include "../common/linux_spi_protocol.h"

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

  return sx * 0.0001 - pow(sy, 3) * 0.00001;
}

void modify_msg_by_gamepad(FLedState *state, LedValuesMessage *msg, GamepadState *gamepad) {
  int8_t left_x, left_y, right_x, right_y = 0;

  double left_stick = stick_value(gamepad->left_x, gamepad->left_y);

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
    state->led1 += left_stick;
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
    state->led2 += left_stick;
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
    state->led3 += left_stick;
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
    state->led4 += left_stick;
  }


  if (gamepad->thumbs & RIGHT_THUMB_UP) {
    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
      state->led1 = 0;
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
      state->led2 = 0;
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
      state->led3 = 0;
    }

    if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
      state->led4 = 0;
    }
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & START_BUTTON) {
    memset(state, 0, sizeof(*state));
  }

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
  } else {
    msg->led1_value = state->led1 * 10;
    msg->led2_value = state->led2 * 10;
    msg->led3_value = state->led3 * 10;
    msg->led4_value = state->led4 * 10;
  }
}

int main(int argc, char *argv[]) {

  int spi_fd = STDOUT_FILENO; // try_open_spi("/dev/spidev0.0", 0);

  char *gamepad_path = DEFAULT_GAMEPAD_PATH;
  if (argc > 1) {
    gamepad_path = argv[1];
  }

  int gamepad_fd = open(gamepad_path, O_RDONLY);

  if (gamepad_fd == -1) {
    report_error(errno);
    return 1;
  }

  GamepadState gamepad = {};

  FLedState state = { 0 };

  for(;;) {
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

    print_gamepad(&gamepad);

    xfer_msg(spi_fd, &msg);
  }
}
