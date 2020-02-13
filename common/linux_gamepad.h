#include <inttypes.h>

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

#define DEFAULT_GAMEPAD_PATH "/dev/hidraw0"

int8_t gamepad_abs_to_rel_axis(uint8_t abs) {
  return abs - 0x80;
}
