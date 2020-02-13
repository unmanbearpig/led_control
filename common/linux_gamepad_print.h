#include "linux_gamepad.h"
#include <stdio.h>

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


  char *select_btn = "";
  if (gamepad->select_start_joystick_buttons_and_shoulders & SELECT_BUTTON) {
    select_btn = "Select";
  }

  char *start_btn = "";
  if (gamepad->select_start_joystick_buttons_and_shoulders & START_BUTTON) {
    start_btn = "Start";
  }

  printf(
         "%04dx%04d %04dx%04d %c%c%c%c %c%c%c%c %s %s\n",
         gamepad_abs_to_rel_axis(gamepad->left_x), gamepad_abs_to_rel_axis(gamepad->left_y),
         gamepad_abs_to_rel_axis(gamepad->right_x), gamepad_abs_to_rel_axis(gamepad->right_y),
         shoulder_left_up, shoulder_left_down,
         shoulder_right_up, shoulder_right_down,
         right_thumb_left, right_thumb_down,
         right_thumb_up, right_thumb_right,
         select_btn, start_btn
         );
  fflush(stdout);
}
