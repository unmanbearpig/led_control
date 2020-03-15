#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/types.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <math.h>
#include <getopt.h>
#include <sys/time.h>

#include <sys/socket.h>
#include <arpa/inet.h>

#include "../common/linux_gamepad.h"
#include "../common/linux_gamepad_print.h"
#include "../common/protocol.h"
#include "../common/linux_util.h"
#include "../common/secrets.h"
#include "../common/gamepad_led_control.h"

void report_error(int err) {
  char *error_str = strerror(err);
  fprintf(stderr, "Error: %s\n", error_str);
}

void print_usage() {
  fprintf(stderr, "TODO: --spi <path to spi dev (default=/dev/spidev0.0)>\n--gamepad <path to gamepad hidraw, default=/dev/hidraw0>\n--verbose - enable verbose output\n");
}

int parse_args(int argc, char *argv[], char **host, int *port, char **gamepad_path, int *btn_map) {
  int argid = 0;

  struct option opts[] =
    {
     { .name = "gamepad", .has_arg = required_argument, .flag = &argid, .val = 'g' },
     { .name = "host", .has_arg = required_argument, .flag = &argid, .val = 'h' },
     { .name = "port", .has_arg = required_argument, .flag = &argid, .val = 'p' },
     { .name = "btn-lu", .has_arg = required_argument, .flag = &argid, .val = 'c' },
     { .name = "btn-ld", .has_arg = required_argument, .flag = &argid, .val = 't' },
     { .name = "btn-ru", .has_arg = required_argument, .flag = &argid, .val = 'r' },
     { .name = "btn-rd", .has_arg = required_argument, .flag = &argid, .val = 'n' },
     { 0, 0, 0, 0 }
    };

  int longindex = 0;
  int ch = 0;

  while( (ch = getopt_long(argc, argv, "s:g:c:t:r:n:v", opts, &longindex)) != -1 ) {
    switch(argid) {
    case 'h':
      *host = optarg;
      break;
    case 'p':
      *port = atoi(optarg);
      break;
    case 'g':
      *gamepad_path = optarg;
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

int open_gamepad(char *gamepad_path) {
  if (0 == strcmp(gamepad_path, "-")) {
    return STDIN_FILENO;
  } else {
    return open(gamepad_path, O_RDONLY | O_ASYNC);
  }
}

int fetch_current_config(GamepadLedControlState *control, int sock, struct sockaddr *sa, size_t sa_size) {
  LedValuesMessage msg = {
                          .magic = LED_VALUES_MESSAGE_MAGIC,
                          .type = LED_READ | LED_CONFIG
  };

  LedValuesMessage input_msg;
  memset(&input_msg, 0, sizeof(input_msg));

  while(1) {
    sendto(sock, &msg, sizeof(msg), 0, sa, sa_size);
    ssize_t bytes_received = recv(sock, &input_msg, sizeof(input_msg), 0);

    if(bytes_received != sizeof(input_msg)) {
      continue;
    }

    if (input_msg.type & LED_CONFIG) {
      control->gamma = input_msg.payload.config.gamma;
      control->pwm_period = input_msg.payload.config.pwm_period;
      return 1;
    }
  }

  return 0;
}

int fetch_current_values(Led *leds, int sock, struct sockaddr *sa, size_t sa_size) {
  LedValuesMessage msg = {
                          .magic = LED_VALUES_MESSAGE_MAGIC,
                          .type = LED_READ,
                          .payload.data.flags = LED_VALUES_FLAG_FLOAT
  };

  LedValuesMessage input_msg;
  memset(&input_msg, 0, sizeof(input_msg));

  while(1) {
    sendto(sock, &msg, sizeof(msg), 0, sa, sa_size);
    ssize_t bytes_received = recv(sock, &input_msg, sizeof(input_msg), 0);

    if(bytes_received != sizeof(input_msg)) {
      continue;
    }

    if (!(input_msg.type & LED_CONFIG) && input_msg.payload.data.flags & LED_VALUES_FLAG_FLOAT) {
      for(int i = 0; i < LED_COUNT; i++) {
        leds[i].value = input_msg.payload.data.values.values_float[i];
      }

      return 1;
    }
  }

  return 0;
}

int fetch_current_state(GamepadLedControlState *control, Led *leds, int sock, struct sockaddr *sa, size_t sa_size) {
  if (!fetch_current_config(control, sock, sa, sa_size)) {
    return 0;
  }

  if (!fetch_current_values(leds, sock, sa, sa_size)) {
    return 0;
  }
  return 1;
}

int try_reopen_gamepad_forever(char *gamepad_path) {
  int gamepad_fd = -1;

  while(gamepad_fd < 0) {
    gamepad_fd = open_gamepad(gamepad_path);
    usleep(100000);
  }

  return gamepad_fd;
}

int main(int argc, char *argv[]) {
  char *gamepad_path = DEFAULT_GAMEPAD_PATH;
  char *host = 0;
  int port = 0;

#ifdef DEFAULT_UDP_PORT
  port = DEFAULT_UDP_PORT;
#endif

  GamepadLedControlState gamepad_led_control;
  init_gamepad_led_control_state(&gamepad_led_control);

  if (!parse_args(argc, argv, &host, &port, &gamepad_path, gamepad_led_control.btn_map)) {
    exit(1);
  }

  if (0 == host) {
    fprintf(stderr, "provide target host as --host\n");
    exit(1);
  }

  if (0 == port) {
    fprintf(stderr, "provide port as --port\n");
    exit(1);
  }

  int gamepad_fd = open_gamepad(gamepad_path);

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

  int sock = socket(AF_INET, SOCK_DGRAM | SOCK_NONBLOCK, IPPROTO_UDP);
  if (sock == -1) {
    fprintf(stderr, "socket failed\n");
    exit(1);
  }

  struct sockaddr_in sa;
  memset(&sa, 0, sizeof(sa));

  sa.sin_family = AF_INET;
  sa.sin_addr.s_addr = inet_addr(host);
  sa.sin_port = htons(port);

  uint8_t buf[sizeof(LedValuesMessage)];

  memset(buf, 0, sizeof(buf));

  int sleep_us = 10;

  uint8_t input_buf[sizeof(LedValuesMessage)];

  if (!fetch_current_state(&gamepad_led_control, leds, sock, (struct sockaddr *)&sa, sizeof(sa))) {
    fprintf(stderr, "Could not fetch current state\n");
    return(1);
  }

  int gamepad_io_error_count = 0;
  for(;;) {
    ssize_t bytes_read = read(gamepad_fd, &gamepad_led_control.gamepad, sizeof(gamepad_led_control.gamepad));

    if (bytes_read != sizeof(gamepad_led_control.gamepad)) {
      int err = errno;
      if (bytes_read < 0) {
        gamepad_io_error_count++;
        if (gamepad_io_error_count > 5) {
          fprintf(stderr, "Waiting to gamepad to reappear...\n");
          close(gamepad_fd);
          gamepad_fd = try_reopen_gamepad_forever(gamepad_path);
        }
      }

      fprintf(stderr, "%ld bytes read, this is an error\n", bytes_read);
      report_error(err);
      continue;
    } else {
      gamepad_io_error_count = 0;
    }

    update_leds_sine(leds);
    modify_msg_by_gamepad(leds, &msg, &gamepad_led_control);

    sendto(sock, &msg, sizeof(msg), 0, (struct sockaddr *)&sa, sizeof(sa));

    ssize_t bytes_received = recv(sock, input_buf, sizeof(input_buf), 0);

    if (bytes_received != -1) {
      if (bytes_received != sizeof(input_buf)) {
        fprintf(stderr, "received %ld bytes instead of %lu\n", bytes_received, (unsigned long)sizeof(input_buf));
      }
      write(STDOUT_FILENO, input_buf, sizeof(input_buf));
    }

    // usleep(sleep_us);
  }
}
