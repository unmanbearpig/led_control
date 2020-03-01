#!/bin/sh

set -e

HOST=$1

# make

rsync -av --delete --exclude ESP8266_RTOS_SDK --exclude .git --exclude xtensa-lx106-elf --exclude esp8266_toolchain --exclude esp8266_wifi_bridge ~/projects/stuff/electronics/led_control/ pi@$HOST:led_control/
