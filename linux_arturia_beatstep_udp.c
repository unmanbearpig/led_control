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

#include <sys/socket.h>
#include <arpa/inet.h>

#include "common/protocol.h"
#include "common/linux_util.h"
#include "common/midi.h"
#include "common/secrets.h"

int verbose = 0;

void report_error(int err) {
  char *error_str = strerror(err);
  fprintf(stderr, "Error: %s\n", error_str);
}

void print_usage() {
  fprintf(stderr, "TODO");
}

int parse_args(int argc, char *argv[], char **host, int *port, char **device_path, int *should_read) {
  int argid = 0;

  struct option opts[] =
    {
     { .name = "device", .has_arg = required_argument, .flag = &argid, .val = 'd' },
     { .name = "host", .has_arg = required_argument, .flag = &argid, .val = 'h' },
     { .name = "port", .has_arg = required_argument, .flag = &argid, .val = 'p' },
     { .name = "verbose", .has_arg = no_argument, .flag = &argid, .val = 'v' },
     { .name = "read", .has_arg = no_argument, .flag = &argid, .val = 'r' },
     { 0, 0, 0, 0 }
    };

  int longindex = 0;
  int ch = 0;

  while( (ch = getopt_long(argc, argv, "s:g:c:t:r:n:vr", opts, &longindex)) != -1 ) {
    switch(argid) {
    case 'h':
      *host = optarg;
      break;
    case 'p':
      *port = atoi(optarg);
      break;
    case 'd':
      *device_path = optarg;
      break;
    case 'v':
      verbose = 1;
      break;
    case 'r':
      *should_read = 1;
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

int get_midi_led_index(MidiMsg *midi_msg) {
  switch(midi_msg->what) {
  case 0x2c:
    return 0;
    break;
  case 0x2d:
    return 1;
    break;
  case 0x2e:
    return 2;
    break;
  case 0x2f:
    return 3;
    break;
  }
  return -1;
}

void modify_msg_by_midi_msg(LedValuesMessage *msg, MidiMsg *midi_msg) {
  uint8_t event_type = midi_msg_status_event(midi_msg);
  int led_index = get_midi_led_index(midi_msg);

  if (led_index < 0) {
    return;
  }

  // uint16_t *led = &msg->payload.data.values.values16[led_index];
  float *led = &msg->payload.data.values.values_float[led_index];

  if (event_type == NOTE_ON) {
    msg->payload.data.amount = 1.0;
    *led = midi_msg->value / 127.0;
  } else if(event_type == POLY_AFTERTOUCH) {
    msg->payload.data.amount /= 4;
    *led = midi_msg->value / 127.0;
    // *led = (midi_msg->value * 2) * (midi_msg->value * 2);
  } else if (event_type == NOTE_OFF) {
    *led = 0.0;
    // *led = 0;
  }
}

int main(int argc, char *argv[]) {
  char *device_path = ""; // DEFAULT_GAMEPAD_PATH;
  char *host = 0;
  int port = 0;
  int should_read = 0;

#ifdef DEFAULT_UDP_PORT
  port = DEFAULT_UDP_PORT;
#endif

  if (!parse_args(argc, argv, &host, &port, &device_path, &should_read)) {
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

  int device_fd = 0;

  if (0 == strcmp(device_path, "-")) {
    device_fd = STDIN_FILENO;
  } else {
    device_fd = open(device_path, O_RDONLY | O_NONBLOCK);
  }

  if (device_fd == -1) {
    fprintf(stderr, "Device: ");
    report_error(errno);
    return 1;
  }

  LedValuesMessage msg =
    {
     .magic = LED_VALUES_MESSAGE_MAGIC,
     .type = LED_WRITE,
     .payload.data = {
                      .flags = LED_VALUES_FLAG_FLOAT | LED_VALUES_FLAG_ADD,
                      .amount = 1.0,
                      .values.values_float = { 0, 0, 0, 0 }
                      }
    };

  if (should_read) {
    msg.type |= LED_READ;
  }

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

  int sleep_us = UDP_SLEEP_US;

  uint8_t input_buf[sizeof(LedValuesMessage)];

  uint8_t midi_input_buf[1024] = { 0 };

  MidiMsg *midi_msg = (MidiMsg *)&midi_input_buf;
  midi_msg_clear(midi_msg);

  for(;;) {
    if (verbose) {
      fprintf(stderr, "\e[1;1H\e[2J"); // clear screen
    }

    ssize_t bytes_read = read(device_fd, midi_input_buf, sizeof(midi_input_buf));

    /* if (bytes_read != sizeof(midi_msg)) { */
    /*   int err = errno; */
    /*   fprintf(stderr, "%ld bytes read, this is an error\n", bytes_read); */
    /*   report_error(err); */
    /*   return 1; */
    /* } */

    modify_msg_by_midi_msg(&msg, midi_msg);

    sendto(sock, &msg, sizeof(msg), 0, (struct sockaddr *)&sa, sizeof(sa));

    if (should_read) {
      ssize_t bytes_received = recv(sock, input_buf, sizeof(input_buf), 0);

      if (bytes_received != -1) {
        if (bytes_received != sizeof(input_buf)) {
          fprintf(stderr, "received %ld bytes instead of %lu\n", bytes_received, (unsigned long)sizeof(input_buf));
        }
        write(STDOUT_FILENO, input_buf, sizeof(input_buf));
      }
    }

    if (verbose) {
      fflush(stderr);
    }

    usleep(sleep_us);
  }
}
