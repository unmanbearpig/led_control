#!/bin/sh

set -e

HOST=$1

rsync -av --delete ~/projects/stuff/electronics/led_control/ pi@$HOST:led_control/
ssh pi@$HOST "pkill a.out" || true
ssh pi@$HOST "~/led_control/test.sh"
