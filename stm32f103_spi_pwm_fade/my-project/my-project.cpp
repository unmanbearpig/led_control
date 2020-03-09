#include <FreeRTOS/FreeRTOS.h>
#include <FreeRTOS/task.h>

#include <string.h>
#include <libopencm3/stm32/rcc.h>
#include <libopencm3/stm32/gpio.h>
#include <libopencm3/cm3/scb.h>
#include <libopencm3/stm32/exti.h>
#include <libopencm3/cm3/nvic.h>
#include <libopencm3/stm32/f1/nvic.h>
#include <libopencm3/stm32/spi.h>
#include <libopencm3/stm32/dma.h>
#include <libopencm3/stm32/timer.h>
#include "../../common/protocol.h"

#include "../../common/stm32_pwm.h"

extern "C" void vApplicationStackOverflowHook( TaskHandle_t xTask, char *pcTaskName );

void vApplicationStackOverflowHook( TaskHandle_t xTask __attribute((unused)), char *pcTaskName __attribute((unused))) {
	for(;;);	// Loop forever here..
}

#define LEDS_COUNT 4
uint16_t leds[LEDS_COUNT] = { 0, 0, 0, 0 };

#define PWM_DELAY_MS 100

void set_leds_to_value(uint16_t *leds, uint16_t value) {
  for(int i = 0; i < LEDS_COUNT; i++) {
    leds[i] = value;
  }
}

static void pwm_fade_task(void *_value) {
  uint16_t value = 0;

  for(;;) {
    for(; value < 0xFFFF; value++) {
      set_leds_to_value(leds, value);
      set_4_leds(leds);
    }

    for(; value > 0; value--) {
      set_leds_to_value(leds, value);
      set_4_leds(leds);
    }

    vTaskDelay(pdMS_TO_TICKS(PWM_DELAY_MS));
  }
}

extern "C" int main(void) {
	rcc_clock_setup_in_hse_8mhz_out_72mhz(); // For "blue pill"
	rcc_periph_clock_enable(RCC_GPIOC);

  rcc_periph_clock_enable(RCC_GPIOA);

  // built-in LED
	gpio_set_mode(GPIOC,
                GPIO_MODE_OUTPUT_2_MHZ,
                GPIO_CNF_OUTPUT_PUSHPULL,
                GPIO13);

  pwm_setup(TIM_OCM_PWM2);

  xTaskCreate(pwm_fade_task, "PWM_FADE", 100, NULL, configMAX_PRIORITIES-1, NULL);
  // xTaskCreate(write_to_spi_dma_task, "START_DMA_TX_VALUE", 100, (void *)&msg, configMAX_PRIORITIES-1, NULL);


	vTaskStartScheduler();

	while(true) {
    gpio_toggle(GPIOC,GPIO13);
  }

	return 0;
}
