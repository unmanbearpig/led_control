PROJECT = stm32f103_spi_pwm_driver
BUILD_DIR = bin

SHARED_DIR = ../my-common-code
CPPFILES = my-project.cpp

# heap_5.c doesn't work, 1-4 work
CFILES = FreeRTOS/heap_4.c FreeRTOS/list.c FreeRTOS/port.c FreeRTOS/queue.c FreeRTOS/tasks.c

# TODO - you will need to edit these two lines!
DEVICE=stm32f103c8t6
OOCD_INTERFACE=stlink-v2
# CS32f103 is a clone of STM32f103 and has a different id
OOCD_TARGET=cs32f1x
# OOCD_TARGET=stm32f1x

# You shouldn't have to edit anything below here.
VPATH += $(SHARED_DIR)
INCLUDES += $(patsubst %,-I%, . $(SHARED_DIR))
OPENCM3_DIR=../../libopencm3

include $(OPENCM3_DIR)/mk/genlink-config.mk
include ../rules.mk
include $(OPENCM3_DIR)/mk/genlink-rules.mk

.PHONY: flash_bmp
flash-bmp: stm32f103_spi_pwm_driver.elf
	gdb-multiarch -nx --batch \
  -ex 'target extended-remote /dev/ttyBmpGdb' \
  -ex 'monitor swdp_scan' \
  -ex 'attach 1' \
  -ex 'load' \
  -ex 'compare-sections' \
  -ex 'kill' \
  *.elf
