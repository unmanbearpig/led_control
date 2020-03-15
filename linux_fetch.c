#include <stdio.h>
#include <getopt.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>

#include <sys/socket.h>
#include <arpa/inet.h>

#include "common/protocol.h"
#include "common/secrets.h"

void print_msg(LedValuesMessage *msg) {
  printf("M%d ", msg->magic);

  char msg_type[4];
  memset(msg_type, '-', 3);
  msg_type[3] = 0;

  if (msg->type & LED_CONFIG) {
    msg_type[0] = 'C';
  }

  if (msg->type & LED_WRITE) {
    msg_type[1] = 'W';
  }

  if (msg->type & LED_READ) {
    msg_type[2] = 'R';
  }

  printf("%s ", msg_type);

  if (msg->type & LED_CONFIG) {
    printf("G=%f ", msg->payload.config.gamma);
    printf("P=%d ", msg->payload.config.pwm_period);
  } else {
    if (msg->payload.data.flags & LED_VALUES_FLAG_FLOAT) {
      printf("f %f %f %f %f A=%f", msg->payload.data.values.values_float[0], msg->payload.data.values.values_float[1], msg->payload.data.values.values_float[2], msg->payload.data.values.values_float[3], msg->payload.data.amount);
    } else {
      printf("d %d %d %d %d", msg->payload.data.values.values16[0], msg->payload.data.values.values16[1], msg->payload.data.values.values16[2], msg->payload.data.values.values16[3]);
    }
  }

  printf("\n");
}

void report_error(int err) {
  char *error_str = strerror(err);
  fprintf(stderr, "Error: %s\n", error_str);
}

void print_usage() {
  printf("\
  --stream[=SLEEP_uS] : streams data continously. Optional sleep in uS between fetches\n\
  --raw : prints raw received bytes\n\
  --config : fetches config, not values\n\
  --float : fetches float values\n\
  --raw : print raw bytes\n\
  --host HOST\n\
  --port PORT\n\
");
}

int parse_args(int argc, char *argv[], char **host, int *port, int *is_raw, int *is_config, int *is_float, int *is_stream, unsigned int *sleep_us) {
  int argid = 0;

  struct option opts[] =
    {
     { .name = "host", .has_arg = required_argument, .flag = &argid, .val = 'h' },
     { .name = "port", .has_arg = required_argument, .flag = &argid, .val = 'p' },
     { .name = "config", .has_arg = no_argument, .flag = &argid, .val = 'c' },
     { .name = "float", .has_arg = no_argument, .flag = &argid, .val = 'f' },
     { .name = "raw", .has_arg = no_argument, .flag = &argid, .val = 'r' },
     { .name = "stream", .has_arg = optional_argument, .flag = &argid, .val = 's' },
     { 0, 0, 0, 0 }
    };

  int longindex = 0;
  int ch = 0;

  while( (ch = getopt_long(argc, argv, "grch:p:s:", opts, &longindex)) != -1 ) {
    switch(argid) {
    case 'h':
      *host = optarg;
      break;
    case 'p':
      *port = atoi(optarg);
      break;
    case 'c':
      *is_config = 1;
      break;
    case 'f':
      *is_float = 1;
      break;
    case 'r':
      *is_raw = 1;
      break;
    case 's':
      *is_stream = 1;
      if (optarg != 0) {
        *sleep_us = atoi(optarg);
      }
      break;
    default:
      fprintf(stderr, "Invalid argument %c\n", argid);
      print_usage();
      return(0);
      break;
    }
  }

  return(1);
}

void make_msg(LedValuesMessage *msg, int is_config, int is_float) {
  memset(msg, 0, sizeof(*msg));

  msg->magic = LED_VALUES_MESSAGE_MAGIC;
  msg->type = LED_READ;
  if (is_config) {
    msg->type |= LED_CONFIG;
    msg->payload.config.flags = 0;
  } else {
    msg->payload.data.flags = 0;
    if (is_float) {
      msg->payload.data.flags |= LED_VALUES_FLAG_FLOAT;
    }
  }
}

int main(int argc, char *argv[]) {
  char *host = 0;
  int port = 0;

#ifdef DEFAULT_UDP_PORT
  port = DEFAULT_UDP_PORT;
#endif

  int is_config = 0;
  int is_float = 0;
  int is_raw = 0;
  int is_stream = 0;
  unsigned int sleep_us = 5000;

  if (!parse_args(argc, argv, &host, &port, &is_raw, &is_config, &is_float, &is_stream, &sleep_us)) {
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

  int sock = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
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
  uint8_t input_buf[sizeof(LedValuesMessage)];

  memset(buf, 0, sizeof(buf));

  LedValuesMessage msg;

  make_msg(&msg, is_config, is_float);

  while(1) {
    if (sendto(sock, &msg, sizeof(msg), 0, (struct sockaddr *)&sa, sizeof(sa)) < 0) {
      int err = errno;
      fprintf(stderr, "sendto: ");
      report_error(err);
      return 1;
    }

    memset(input_buf, 0, sizeof(input_buf));
    ssize_t bytes_received = recv(sock, input_buf, sizeof(input_buf), 0);

    if (bytes_received < 0) {
      int err = errno;
      fprintf(stderr, "receiving msg: ");
      report_error(err);
      return 1;
    }

    if (bytes_received != sizeof(input_buf)) {
      fprintf(stderr, "received wrong number of bytes %ld instead of %lu\n", bytes_received, (unsigned long)sizeof(input_buf));
      return 1;
    }

    LedValuesMessage *input_msg = (LedValuesMessage *)input_buf;

    if ((msg.type & LED_CONFIG) != (input_msg->type & LED_CONFIG)) {
      continue;
    }

    if (is_raw) {
      write(STDOUT_FILENO, input_buf, sizeof(input_buf));
    } else {
      print_msg(input_msg);
    }

    if(!is_stream) {
      break;
    }

    if (sleep_us > 0) {
      usleep(sleep_us);
    }
  }

  return 0;
}
