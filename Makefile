default: linux stm32

spi_fade_test: flash fade_test

fade_test: bin/spi_fade_test
	bin/spi_fade_test | grep '<' | grep -v '0000 0000 0000 0000 0000'

spi_echo_test: flash_stm32f103_spi_echo fade_test

linux: bin/spi_fade_test bin/gamepad bin/spi_pipe bin/fade_pipe bin/gamepad_spi bin/udp_spi bin/udp_gamepad

bin/spi_pipe: bin linux_spi_pipe/main.c
	gcc linux_spi_pipe/main.c -o bin/spi_pipe

bin/fade_pipe: bin linux_fade_pipe/main.c
	gcc linux_fade_pipe/main.c -o bin/fade_pipe

bin/spi_fade_test: bin linux_spi_fade_test/main.c
	gcc linux_spi_fade_test/main.c -o bin/spi_fade_test

bin/gamepad: bin linux_gamepad/main.c
	gcc linux_gamepad/main.c -o bin/gamepad

bin/gamepad_spi: bin linux_gamepad_spi/main.c
	gcc linux_gamepad_spi/main.c -o bin/gamepad_spi -lm

bin/udp_spi: bin linux_udp_spi/main.c
	gcc linux_udp_spi/main.c -o bin/udp_spi

bin/udp_gamepad: bin linux_udp_gamepad/main.c
	gcc linux_udp_gamepad/main.c -o bin/udp_gamepad

bin:
	mkdir bin

stm32: stm32f103_spi_pwm_driver stm32f103_spi_echo

stm32f103_spi_pwm_driver: stm32f103_spi_pwm_driver/my-project/bin/my-project.o

stm32f103_spi_pwm_driver/my-project/bin/my-project.o:
	make -C stm32f103_spi_pwm_driver/my-project

flash_stm32f103_spi_echo: stm32f103_spi_echo
	make flash -C stm32f103_spi_echo/my-project

stm32f103_spi_echo: stm32f103_spi_echo/my-project/bin/my-project.o

stm32f103_spi_echo/my-project/bin/my-project.o:
	make -C stm32f103_spi_echo/my-project

clean:
	rm -rf bin
	make clean -C stm32f103_spi_pwm_driver/my-project

flash: stm32f103_spi_pwm_driver
	make flash -C stm32f103_spi_pwm_driver/my-project
