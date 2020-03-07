#include <arpa/inet.h>

#include "protocol.h"

int send_msg(int sock, struct sockaddr_in *sa, LedValuesMessage *msg) {
  return sendto(sock, msg, sizeof(*msg), 0, (struct sockaddr *)sa, sizeof(*sa));
}
