#include <unistd.h>
#include <getopt.h>
#include <stdio.h>
#include <stdlib.h>

void print_usage() {
  fprintf(stderr, "--spi <path to spi dev (default=/dev/spidev0.0)>\n--gamepad <path to gamepad hidraw, default=/dev/hidraw0>");
}

int main(int argc, char *argv[]) {
  int argid = 0;

  struct option opts[] =
    {
     { .name = "spi", .has_arg = required_argument, .flag = &argid, .val = 's', },
     { .name = "gamepad", .has_arg = required_argument, .flag = &argid, .val = 'g', },
     { 0, 0, 0, 0 }
    };

  int longindex = 0;

  char *spi_path = "/dev/spidev0.0";
  char *gamepad_path = "/dev/hidraw0";

  int ch = 0;
  while( (ch = getopt_long(argc, argv, "s:g:", opts, &longindex)) != -1 ) {
    switch(argid) {
    case 's':
      spi_path = optarg;
      break;
    case 'g':
      gamepad_path = optarg;
      break;
    default:
      fprintf(stderr, "Invalid argument\n");
      print_usage();
      exit(1);
      break;
    }
  }

  printf("spi: %s\n", spi_path);
  printf("gamepad: %s\n", gamepad_path);
}
