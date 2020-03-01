#!/bin/sh

set -e

HOST=$1

./sync.sh $HOST

ssh pi@$HOST "pkill spi_fade_test" || true
ssh pi@$HOST "make $2 -C ~/led_control"

# ssh pi@$HOST "make clean -C ~/led_control"
# ssh pi@$HOST "cd ~/led_control && ./test.sh"

echo '---- done ----'
