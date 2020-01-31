default: linux stm32

fade_test: bin/spi_fade_test flash
	bin/spi_fade_test

linux: bin/spi_fade_test bin/gamepad bin/spi_pipe bin/fade_pipe

bin/spi_pipe: bin linux_spi_pipe/main.c
	gcc linux_spi_pipe/main.c -o bin/spi_pipe

bin/fade_pipe: bin linux_fade_pipe/main.c
	gcc linux_fade_pipe/main.c -o bin/fade_pipe

bin/spi_fade_test: bin linux_spi_fade_test/main.c
	gcc linux_spi_fade_test/main.c -o bin/spi_fade_test

bin/gamepad: bin linux_gamepad/main.c
	gcc linux_gamepad/main.c -o bin/gamepad

bin:
	mkdir bin

stm32: stm32f103_spi_pwm_driver

stm32f103_spi_pwm_driver: stm32f103_spi_pwm_driver/my-project/bin/my-project.o

stm32f103_spi_pwm_driver/my-project/bin/my-project.o:
	make -C stm32f103_spi_pwm_driver/my-project

clean:
	rm -rf bin
	make clean -C stm32f103_spi_pwm_driver/my-project

flash: stm32f103_spi_pwm_driver
	make flash -C stm32f103_spi_pwm_driver/my-project
