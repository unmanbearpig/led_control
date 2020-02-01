#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/types.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>

#define SHOULDERS 0x0F
#define SHOULDER_LEFT_UP 0x01
#define SHOULDER_LEFT_DOWN 0x04
#define SHOULDER_RIGHT_UP 0x02
#define SHOULDER_RIGHT_DOWN 0x08

#define RIGHT_THUMB 0xF0
#define RIGHT_THUMB_LEFT 0x80
#define RIGHT_THUMB_RIGHT 0x20
#define RIGHT_THUMB_UP 0x10
#define RIGHT_THUMB_DOWN 0x40

#define LEFT_JOYSTICK_BUTTON 0x40
#define RIGHT_JOYSTICK_BUTTON 0x80
#define SELECT_BUTTON 0x10
#define START_BUTTON 0x20


typedef struct {
  uint8_t left_x;
  uint8_t left_y;
  uint8_t reserved_1;
  uint8_t right_x;
  uint8_t right_y;
  uint8_t thumbs; // weird, = 0x0F when nothing is pressed down
  uint8_t select_start_joystick_buttons_and_shoulders;
  uint8_t reserved_2;
} GamepadState;

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
         "%02Xx%02X %02Xx%02X %c%c%c%c %c%c%c%c\n",
         gamepad->left_x, gamepad->left_y,
         gamepad->right_x, gamepad->right_y,
         shoulder_left_up, shoulder_left_down,
         shoulder_right_up, shoulder_right_down,
         right_thumb_left, right_thumb_down,
         right_thumb_up, right_thumb_right
         );
  fflush(stdout);
}

int main(int argc __attribute((unused)), char *argv[] __attribute((unused))) {
  int gamepad_fd = open("/dev/hidraw0", O_RDONLY);

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
