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
#include "../common/gamepad_led_control.h"

int verbose = 0;

void report_error(int err) {
  char *error_str = strerror(err);
  fprintf(stderr, "Error: %s\n", error_str);
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

/* double adjustment_exp = DEFAULT_ADJUSTMENT_EXP; */

void print_usage() {
  fprintf(stderr, "TODO: --spi <path to spi dev (default=/dev/spidev0.0)>\n--gamepad <path to gamepad hidraw, default=/dev/hidraw0>\n--verbose - enable verbose output\n");
}

int parse_args(int argc, char *argv[], char **gamepad_path, uint32_t *speed, useconds_t *sleep_us, int *btn_map) {
  int argid = 0;

  struct option opts[] =
    {
     { .name = "gamepad", .has_arg = required_argument, .flag = &argid, .val = 'g' },
     { .name = "speed", .has_arg = required_argument, .flag = &argid, .val = 's' },
     { .name = "sleep", .has_arg = required_argument, .flag = &argid, .val = 'z' },
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
    case 'g':
      *gamepad_path = optarg;
      break;
    case 's':
      *speed = atoi(optarg);
      break;
    case 'z':
      *sleep_us = atoi(optarg);
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
  printf("hello\n");

  char *gamepad_path = DEFAULT_GAMEPAD_PATH;
  char *spi_path = "/dev/spidev0.0";
  uint32_t speed = 0;
  useconds_t sleep_us = 0;
  GamepadLedControlState gamepad_led_control;


  printf("111111\n");
  init_gamepad_led_control_state(&gamepad_led_control);

printf("222222\n");

  if (!parse_args(argc, argv, &gamepad_path, &speed, &sleep_us, gamepad_led_control.btn_map)) {
    exit(1);
  }

  printf("333333\n");

  int spi_fd = try_open_spi(spi_path, speed);
  int gamepad_fd = 0;

  if (0 == strcmp(gamepad_path, "-")) {
    gamepad_fd = STDIN_FILENO;
  } else {
    gamepad_fd = open(gamepad_path, O_RDONLY | O_ASYNC);
  }

  printf("444444\n");

  if (gamepad_fd == -1) {
    fprintf(stderr, "Gamepad: ");
    report_error(errno);
    return 1;
  }

  Led leds[4];
  memset(leds, 0, sizeof(leds));

  LedValuesMessage msg =
    {
     .magic = LED_VALUES_MESSAGE_MAGIC,
     .type = LED_WRITE | LED_READ,
     .payload.data = {
                      .flags = LED_VALUES_FLAG_FLOAT,
                      .amount = 1.0,
                      .values.values_float = { 0, 0, 0, 0 }
                      }
    };

  uint8_t buf[sizeof(LedValuesMessage)];

  memset(buf, 0, sizeof(buf));

  LedValuesMessage input_msg;
  memset(&input_msg, 0, sizeof(input_msg));

  for(;;) {
    if (verbose) {
      fprintf(stderr, "\e[1;1H\e[2J"); // clear screen
    }

    ssize_t bytes_read = read(gamepad_fd, &gamepad_led_control.gamepad, sizeof(gamepad_led_control.gamepad));

    if (bytes_read != sizeof(gamepad_led_control.gamepad)) {
      int err = errno;
      fprintf(stderr, "%ld bytes read, this is an error\n", bytes_read);
      report_error(err);
      return 1;
    }

    update_leds_sine(leds);
    modify_msg_by_gamepad(leds, &msg, &gamepad_led_control);

    if (verbose) {
      print_gamepad(&gamepad_led_control.gamepad);
    }

    // write(STDOUT_FILENO, &msg, sizeof(msg));
    xfer_msg_2(spi_fd, &msg, &input_msg, verbose);
    write(STDOUT_FILENO, &input_msg, sizeof(input_msg));

    if (verbose) {
      fflush(stderr);
    }

    if (sleep_us > 0) {
      usleep(sleep_us);
    }
  }
}
