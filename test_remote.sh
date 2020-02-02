#!/bin/sh

set -e

HOST=$1

make clean
make

rsync -av --delete ~/projects/stuff/electronics/led_control/ pi@$HOST:led_control/
ssh pi@$HOST "pkill spi_fade_test" || true
ssh pi@$HOST "make clean -C ~/led_control"
ssh pi@$HOST "cd ~/led_control && ./test.sh"

echo '---- done ----'
