#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <inttypes.h>
#include <string.h>
#include <math.h>
#include "../common/protocol.h"

#define SAMPLE_COUNT 10000

double get_avg_amplitude(int16_t *buf, int count) {
  double avg = 0;
  double max = 0;

  int64_t avg64 = 0;

  int16_t imin = 32000;
  int16_t imax = -32000;

  for(int i = 0; i < count; i++) {
    // int16_t sval = buf[i] - 0x8000;
    int16_t sval = buf[i];
    int16_t absval = abs(sval);
    double fabsval = absval;

    if (sval > imax) {
      imax = absval;
    }

    if (sval < imin) {
      imin = sval;
    }

    avg64 += sval;

    if (fabsval > max) {
      max = fabsval;
    }

    avg += pow(fabsval, 2);
  }

  avg = sqrt(avg / count);
  // avg = pow(avg / count, 10);
  // printf("%e\n", max);

  //printf("%e\n", avg);

  return avg;
}

int main(int argc, char *argv[]) {
  int16_t buf[SAMPLE_COUNT];
  memset(buf, 0, sizeof(buf));

  double avg = 0;

  LedValuesMessage msg;
  set_valid_msg_magic(&msg);

  double min_avg = 99999999999999.0;

  double avg_avg = 0.0;
  for(uint64_t cnt = 0;;cnt++) {
    read(STDIN_FILENO, buf, sizeof(buf));
    avg = get_avg_amplitude(buf, SAMPLE_COUNT);

    if (avg < min_avg) {
      min_avg = avg;
    }

    avg = avg - min_avg;

    if (cnt == 0) {
      avg_avg = avg;
    } else {
      avg_avg = ((avg_avg * (cnt -1)) + avg) / cnt;
    }

    // uint16_t uint_avg = avg * 65535.0;

    uint16_t uint_avg = avg;
    set_all_msg_values(&msg, uint_avg);

    write(STDOUT_FILENO, &msg, sizeof(msg));

    // if ((cnt % 2) == 0)  {
      // print_msg(&msg);
      // printf("%e %e %e %e %ld\n", min_avg, avg, avg_avg, fabs(avg - avg_avg), cnt);
    // }
    // printf("%04X\n", uint_avg);
  }
}
