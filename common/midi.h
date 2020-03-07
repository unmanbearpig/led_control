#include <inttypes.h>

enum {
      NOTE_OFF = 0x8,
      NOTE_ON = 0x9,
      POLY_AFTERTOUCH = 0xA,
      CONTROL_CHANGE = 0xB,
      PROGRAM_CHANGE = 0xC,
      CHANNEL_AFTERTOUCH = 0xD,
      PITCH_WHEEL = 0xE
} midi_status_event;

typedef struct __attribute__((__packed__)) {
  uint8_t status;
  uint8_t what;
  uint8_t value;
} MidiMsg;

uint8_t midi_msg_status_event(MidiMsg *msg) {
  return msg->status >> 4;
}

void midi_msg_clear(MidiMsg *msg) {
  msg->status = 0;
  msg->what = 0;
  msg->value = 0;
}

/* uint8_t midi_msg_channel(MidiMsg *msg) { */
/*   return msg->status & 0x0F; */
/* } */



/* Note Off 	0x8 	Channel */
/* 0x0..0xF 	Note Number 	Velocity */
/* Note On 	0x9 	Note Number 	Velocity */
/* Polyphonic Key Pressure (Aftertouch) 	0xA 	Note Number 	Pressure */
/* Control Change 	0xB 	Controller Number 	Value */
/* Program Change 	0xC 	Program Number 	 */
/* Channel Pressure (Aftertouch) 	0xD 	Pressure 	 */
/* Pitch Wheel 	0xE 	LSB 	MSB  */
