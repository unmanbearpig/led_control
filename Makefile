default: linux stm32 esp8266

spi_fade_test: flash fade_test

fade_test: bin/spi_fade_test
	bin/spi_fade_test | grep '<' | grep -v '0000 0000 0000 0000 0000'

spi_echo_test: flash_stm32f103_spi_echo fade_test

linux: bin/spi_fade_test bin/gamepad bin/spi_pipe bin/fade_pipe bin/gamepad_spi bin/udp_spi bin/udp_gamepad bin/sine bin/set_value bin/udp_xfer

bin/spi_pipe: linux_spi_pipe/main.c
	[ -d bin ] || mkdir bin
	gcc linux_spi_pipe/main.c -o bin/spi_pipe

bin/fade_pipe: linux_fade_pipe/main.c
	[ -d bin ] || mkdir bin
	gcc linux_fade_pipe/main.c -o bin/fade_pipe

bin/spi_fade_test: linux_spi_fade_test/main.c
	[ -d bin ] || mkdir bin
	gcc linux_spi_fade_test/main.c -o bin/spi_fade_test

bin/gamepad: linux_gamepad/main.c
	[ -d bin ] || mkdir bin
	gcc linux_gamepad/main.c -o bin/gamepad

bin/gamepad_spi: linux_gamepad_spi/main.c
	[ -d bin ] || mkdir bin
	gcc linux_gamepad_spi/main.c -o bin/gamepad_spi -lm

bin/udp_spi: linux_udp_spi/main.c
	[ -d bin ] || mkdir bin
	gcc linux_udp_spi/main.c -o bin/udp_spi

bin/udp_gamepad: linux_udp_gamepad/main.c
	[ -d bin ] || mkdir bin
	gcc linux_udp_gamepad/main.c -o bin/udp_gamepad  -lm

bin/sine: linux_sine/main.c
	[ -d bin ] || mkdir bin
	gcc linux_sine/main.c -o bin/sine -lm

bin/set_value: linux_set_value/main.c
	[ -d bin ] || mkdir bin
	gcc linux_set_value/main.c -o bin/set_value

bin/udp_xfer: linux_udp_xfer/main.c
	[ -d bin ] || mkdir bin
	gcc linux_udp_xfer/main.c -o bin/udp_xfer

esp8266: esp8266_wifi_bridge

ESP8266_TOOLCHAIN_BIN = esp8266_toolchain/xtensa-lx106-elf/bin
ESP8266_TOOLCHAIN=$(ESP8266_TOOLCHAIN_BIN)/.extracted
ESP8266_TOOLCHAIN_URL = https://dl.espressif.com/dl/xtensa-lx106-elf-linux64-1.22.0-100-ge567ec7-5.2.0.tar.gz
ESP8266_TOOLCHAIN_SHA256 = 706a02853759c2f85d912f68df4f5b4566ecb41422de5afe35a45d064eb8e494
LED_CONTROL_PATH=$(dir $(realpath $(firstword $(MAKEFILE_LIST))))
esp8266_toolchain:
	mkdir esp8266_toolchain

$(ESP8266_TOOLCHAIN): esp8266_toolchain/xtensa-lx106-elf-linux64.tar.gz
	test $(ESP8266_TOOLCHAIN_SHA256) = "$$(sha256sum esp8266_toolchain/xtensa-lx106-elf-linux64.tar.gz | cut -f 1 -d ' ')"
	tar -zxvf esp8266_toolchain/xtensa-lx106-elf-linux64.tar.gz -C esp8266_toolchain/
	touch $(ESP8266_TOOLCHAIN)

esp8266_toolchain/xtensa-lx106-elf-linux64.tar.gz:
	mkdir esp8266_toolchain; true
	curl "$(ESP8266_TOOLCHAIN_URL)" -o esp8266_toolchain/xtensa-lx106-elf-linux64.tar.gz

esp8266_wifi_bridge: esp8266_wifi_bridge/build/esp8266_wifi_bridge.elf

esp8266_wifi_bridge/build/esp8266_wifi_bridge.elf: export CONFIG_SDK_TOOLPREFIX = $(LED_CONTROL_PATH)$(ESP8266_TOOLCHAIN_BIN)/xtensa-lx106-elf-
esp8266_wifi_bridge/build/esp8266_wifi_bridge.elf: export IDF_PATH = $(realpath $(LED_CONTROL_PATH)/ESP8266_RTOS_SDK)
esp8266_wifi_bridge/build/esp8266_wifi_bridge.elf: $(ESP8266_TOOLCHAIN) esp8266_wifi_bridge/main/*.c
	./make_shit_esp_project.sh esp8266_wifi_bridge

stm32: stm32f103_spi_pwm_driver stm32f103_spi_pwm_fade stm32f103_spi_echo

stm32f103_spi_pwm_fade: stm32f103_spi_pwm_fade/my-project/bin/my-project.o

stm32f103_spi_pwm_driver: stm32f103_spi_pwm_driver/my-project/bin/my-project.o

stm32f103_spi_pwm_fade/my-project/bin/my-project.o:
	make -C stm32f103_spi_pwm_fade/my-project

stm32f103_spi_pwm_driver/my-project/bin/my-project.o:
	make -C stm32f103_spi_pwm_driver/my-project

flash_stm32f103_spi_echo: stm32f103_spi_echo
	make flash -C stm32f103_spi_echo/my-project

stm32f103_spi_echo: stm32f103_spi_echo/my-project/bin/my-project.o

stm32f103_spi_echo/my-project/bin/my-project.o:
	make -C stm32f103_spi_echo/my-project

.PHONY: clean
clean:
	rm -rf bin
	make clean -C stm32f103_spi_pwm_driver/my-project
	make clean -C stm32f103_spi_pwm_fade/my-project
	make clean -C esp8266_wifi_bridge

.PHONY: flash
flash: stm32f103_spi_pwm_driver
	make flash -C stm32f103_spi_pwm_driver/my-project

.PHONY: esp8266-flash
esp8266-flash: esp8266
	make flash -C esp8266_wifi_bridge

.PHONY: esp8266-monitor
esp8266-monitor:
	@echo "C-] to exit\n"
	make monitor -C esp8266_wifi_bridge
