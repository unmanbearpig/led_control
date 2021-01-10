PROJECT = try-usb
BUILD_DIR = bin

SHARED_DIR = ../my-common-code
CFILES = my-project.c
CFILES += api.c
AFILES += api-asm.S

# TODO - you will need to edit these two lines!
DEVICE=stm32f103c8t6
OOCD_INTERFACE=stlink-v2
OOCD_TARGET=cs32f1x

# DEVICE=stm32f407vgt6
# OOCD_FILE = board/stm32f4discovery.cfg

# You shouldn't have to edit anything below here.
VPATH += $(SHARED_DIR)
INCLUDES += $(patsubst %,-I%, . $(SHARED_DIR))
OPENCM3_DIR=../libopencm3

include $(OPENCM3_DIR)/mk/genlink-config.mk
include ../rules.mk
include $(OPENCM3_DIR)/mk/genlink-rules.mk

.PHONY: flash-bmp debug
flash-bmp: try-usb.elf
	gdb -nx --batch \
  -ex 'target extended-remote /dev/ttyBmpGdb' \
  -ex 'monitor swdp_scan' \
  -ex 'attach 1' \
  -ex 'load' \
  -ex 'compare-sections' \
  -ex 'kill' \
  *.elf

debug:
	gdb \
	-ex 'target extended-remote /dev/ttyBmpGdb' \
  -ex 'monitor swdp_scan' \
  -ex 'attach 1' \
	try-usb.elf
