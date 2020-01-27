#!/bin/sh

set -e

make flash -C "~/led_control/stm32f103_spi_pwm_driver/my-project"
gcc ~/led_control/linux_spi/main.c -o ~/led_control/linux_spi/a.out
~/led_control/linux_spi/a.out
