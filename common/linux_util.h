#pragma once

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

void pabort(const char *s)
{
	perror(s);
	abort();
}

ssize_t read_all(int fd, void *_buf, ssize_t len) {
  ssize_t bytes_read = 0;
  char *buf = _buf;

  while(bytes_read < len) {
    ssize_t tmp_bytes_read = read(fd, buf + bytes_read, len - bytes_read);
    if (tmp_bytes_read < 0) {
      return tmp_bytes_read;
    }

    bytes_read += tmp_bytes_read;
  }

  return bytes_read;
}
