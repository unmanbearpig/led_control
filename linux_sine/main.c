#include <sys/time.h>
#include <unistd.h>
#include <math.h>
#include <string.h>
#include <stdio.h>
#include "../common/protocol.h"

int main(int argc, char *argv[]) {
  struct timeval tv;
  memset(&tv, 0, sizeof(tv));

  LedValuesMessage msg;
  set_valid_msg_magic(&msg);

  double freq = 0.000005;
  double amplitude = 0.01;

  for(;;) {
    if (-1 == gettimeofday(&tv, NULL)) {
      fprintf(stderr, "gettimeofday error\n");
      return(1);
    }

    uint64_t t = tv.tv_sec * 1000000 + tv.tv_usec;

    double sine_val = sin(t * freq) * amplitude;

    // set_all_msg_values(&msg, (sin(t * freq) * amplitude) * 65535);
    printf("%f\n", sine_val);
    // write(STDOUT_FILENO, &msg, sizeof(msg));
  }

  return 0;
}
