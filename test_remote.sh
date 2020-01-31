#!/bin/sh

set -e

HOST=$1

rsync -av --delete ~/projects/stuff/electronics/led_control/ pi@$HOST:led_control/
ssh pi@$HOST "pkill spi_fade_test" || true
ssh pi@$HOST "cd ~/led_control && ./test.sh"

echo '---- done ----'
