#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/types.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include "../common/linux_gamepad.h"
#include "../common/linux_gamepad_print.h"

void report_error(int err) {
  char *error_str = strerror(err);
  fprintf(stderr, "Error: %s\n", error_str);
}

int main(int argc, char *argv[]) {
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
