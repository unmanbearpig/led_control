#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/types.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include "../common/linux_gamepad.h"

void report_error(int err) {
  char *error_str = strerror(err);
  fprintf(stderr, "Error: %s\n", error_str);
}

void print_gamepad(GamepadState *gamepad) {
  char shoulder_left_up = ' ';
  char shoulder_left_down = ' ';
  char shoulder_right_up = ' ';
  char shoulder_right_down = ' ';

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_UP) {
    shoulder_left_up = '^';
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_LEFT_DOWN) {
    shoulder_left_down = '_';
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_UP) {
    shoulder_right_up = '^';
  }

  if (gamepad->select_start_joystick_buttons_and_shoulders & SHOULDER_RIGHT_DOWN) {
    shoulder_right_down = '_';
  }

  char right_thumb_left = ' ';
  char right_thumb_down = ' ';
  char right_thumb_up = ' ';
  char right_thumb_right = ' ';

  if (gamepad->thumbs & RIGHT_THUMB_LEFT) {
    right_thumb_left = '<';
  }

  if (gamepad->thumbs & RIGHT_THUMB_DOWN) {
    right_thumb_down = 'V';
  }

  if (gamepad->thumbs & RIGHT_THUMB_UP) {
    right_thumb_up = '^';
  }

  if (gamepad->thumbs & RIGHT_THUMB_RIGHT) {
    right_thumb_right = '>';
  }

  printf(
         "%04dx%04d %04dx%04d %c%c%c%c %c%c%c%c\n",
         gamepad_abs_to_rel_axis(gamepad->left_x), gamepad_abs_to_rel_axis(gamepad->left_y),
         gamepad_abs_to_rel_axis(gamepad->right_x), gamepad_abs_to_rel_axis(gamepad->right_y),
         shoulder_left_up, shoulder_left_down,
         shoulder_right_up, shoulder_right_down,
         right_thumb_left, right_thumb_down,
         right_thumb_up, right_thumb_right
         );
  fflush(stdout);
}

#define DEFAULT_GAMEPAD_PATH "/dev/hidraw0"

int main(int argc __attribute((unused)), char *argv[] __attribute((unused))) {
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

  for(;;) {
    ssize_t bytes_read = read(gamepad_fd, &gamepad, sizeof(gamepad));

    if (bytes_read != sizeof(gamepad)) {
      int err = errno;
      fprintf(stderr, "%ld bytes read, this is an error\n", bytes_read);
      report_error(err);
      return 1;
    }
    print_gamepad(&gamepad);
  }
}
