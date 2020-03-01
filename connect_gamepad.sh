#!/bin/sh

make linux
while true; do test -c /dev/hidraw0 && bin/gamepad_udp --gamepad /dev/hidraw0 --host 127.0.0.1 >/dev/null ; sleep 1; done
