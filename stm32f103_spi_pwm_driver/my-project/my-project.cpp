#include <FreeRTOS/FreeRTOS.h>
#include <FreeRTOS/task.h>

#include <libopencm3/stm32/rcc.h>
#include <libopencm3/stm32/gpio.h>
#include <libopencm3/cm3/scb.h>
#include <libopencm3/stm32/exti.h>
#include <libopencm3/cm3/nvic.h>
#include <libopencm3/stm32/f1/nvic.h>
#include <libopencm3/stm32/spi.h>
#include <libopencm3/stm32/dma.h>
#include <libopencm3/stm32/timer.h>

#include "pwm.h"

extern "C" void vApplicationStackOverflowHook( TaskHandle_t xTask, char *pcTaskName );

void vApplicationStackOverflowHook( TaskHandle_t xTask __attribute((unused)), char *pcTaskName __attribute((unused))) {
	for(;;);	// Loop forever here..
}

uint32_t led_delay_ms = 1500;

// static void task1(void *args __attribute((unused))) {
// 	for (;;) {
// 		gpio_toggle(GPIOC,GPIO13);
// 		vTaskDelay(pdMS_TO_TICKS(led_delay_ms));
// 	}
// }

struct LedFadeValue {
  tim_oc_id channel; // e.g. TIM_OC2
  uint16_t value;
  uint16_t min_value;
  uint16_t max_value;
  int16_t step;
};

static void fade_task(void *_value) {
  struct LedFadeValue *led_fade_value = (LedFadeValue *)_value;

  for (;;) {
    timer_set_oc_value(TIM1, led_fade_value->channel, (0xFFFF - led_fade_value->value));
    led_fade_value->value += led_fade_value->step;

    if (led_fade_value->value >= led_fade_value->max_value) {
      led_fade_value->value = led_fade_value->min_value;
    }

    vTaskDelay(pdMS_TO_TICKS(50));
  }
}

struct LedValue {
  tim_oc_id channel; // e.g. TIM_OC2
  uint16_t value;
};

static void set_led_value_task(void *_value) {
  struct LedValue *led_value = (LedValue *)_value;

  for(;;) {
    timer_set_oc_value(TIM1, led_value->channel, (0xFFFF-led_value->value));
    vTaskDelay(pdMS_TO_TICKS(10));
  }
}

// SPI1
// clock: RCC_SPI1
// GPIOs:
//   PA4: SPI1_NSS
//   PA5: SPI1_SCK
//   PA6/PB4?: SPI1_MISO
//   PA7/PB5?: SPI1_MOSI

// spi configuration
//   DFF: 8 or 16 bit frame format
//
void spi_setup() {
  rcc_periph_clock_enable(RCC_SPI1);

  gpio_set_mode(GPIOA,
                GPIO_MODE_OUTPUT_50_MHZ,
                GPIO_CNF_OUTPUT_ALTFN_PUSHPULL,
                GPIO6);
	gpio_set_mode(GPIOA,
                GPIO_MODE_INPUT,
                GPIO_CNF_INPUT_FLOAT,
                GPIO4|GPIO5|GPIO7);
  // // ???
  // gpio_set_af(GPIOA, GPIO_AF5, GPIO5 | GPIO6 | GPIO7);

  spi_disable(SPI1);
  spi_reset(SPI1);

  // spi_init_master(SPI1,
  //                 SPI_CR1_BAUDRATE_FPCLK_DIV_2,
  //                 SPI_CR1_CPOL_CLK_TO_0_WHEN_IDLE,
  //                 SPI_CR1_CPHA_CLK_TRANSITION_1,
  //                 SPI_CR1_DFF_8BIT,
  //                 SPI_CR1_MSBFIRST);

  spi_set_slave_mode(SPI1);
  // spi_set_receive_only_mode(SPI1);

  spi_disable_crc(SPI1);

  // low when idle
  spi_set_clock_polarity_0(SPI1);

  // which edge is that?
  // esp has posedge
  // rising = leading
  // 0 = rising, I assume linux has it by default (?)
  spi_set_clock_phase_0(SPI1);
  spi_send_msb_first(SPI1);
  spi_enable(SPI1);
  // spi_reset(SPI1);
  spi_enable_rx_buffer_not_empty_interrupt(SPI1);
  spi_enable_error_interrupt(SPI1);
}



// https://www.rhye.org/post/stm32-with-opencm3-2-spi-and-dma/
void spi_dma_setup() {
  rcc_periph_clock_enable(RCC_DMA1);

  // not sure what it does and where it should be
  dma_channel_reset(DMA1, DMA_CHANNEL2);
  dma_set_peripheral_address(DMA1, DMA_CHANNEL2, (uint32_t)&SPI1_DR);

 	nvic_set_priority(NVIC_DMA1_CHANNEL2_IRQ, 0);
	nvic_enable_irq(NVIC_DMA1_CHANNEL2_IRQ);
}

void read_from_spi_dma(uint16_t *rx_buf, uint32_t rx_len) {
  uint32_t dma = DMA1;
  uint8_t channel = DMA_CHANNEL2;

  dma_set_memory_address(dma, channel, (uint32_t)rx_buf);
  // linux doesn't support 16 bits?
  dma_set_memory_size(dma, channel, DMA_CCR_MSIZE_8BIT);
  dma_set_peripheral_size(dma, channel, DMA_CCR_PSIZE_8BIT);
  dma_set_number_of_data(dma, channel, rx_len);
  dma_set_priority(dma, channel, DMA_CCR_PL_LOW); // whatever I guess
  dma_set_read_from_peripheral(dma, channel);

  dma_enable_memory_increment_mode(dma, channel);
  // dma_disable_memory_increment_mode(dma, channel);

  dma_enable_channel(dma, channel);
  dma_enable_circular_mode(dma, channel);

  spi_enable_rx_dma(SPI1);

  // dma_enable_transfer_complete_interrupt(dma, channel);
  // dma_enable_half_transfer_interrupt(dma, channel);
  // dma_enable_transfer_error_interrupt(dma, channel);
}

struct LedFadeValue led_fade_values[4] =
  { { TIM_OC1, 0, 0x0000, 0x00FF, 10 },
    { TIM_OC2, 0, 0x0000, 0x00FF, 13 },
    { TIM_OC3, 0, 0x0000, 0x00FF,  5 },
    { TIM_OC4, 0, 0x0000, 0x00FF,  7 } };

struct LedValue led_value =
  { TIM_OC3, 0x0030 };

void spi1_isr() {
  // led_value.value = 0xFFFF;
  // gpio_toggle(GPIOC,GPIO13);

}

void dma_channel2_isr() {
  //xTaskCreate(set_led_value_task, "SET_LED_VALUE", 100, (void *)&led_value, configMAX_PRIORITIES-1, NULL);
  // led_value.value = 0xFFFF;
  // gpio_toggle(GPIOC,GPIO13);
  // dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_TCIF);

}

// static void read_spi_task(void *args __attribute((unused))) {
//   for (;;) {
//     led_value.value = spi_read(SPI1);
//     vTaskDelay(pdMS_TO_TICKS(5));
//   }
// }

// static void read_spi_dma_task(void *args __attribute((unused))) {
//   for (;;) {
//     read_from_spi_dma(&led_value.value, sizeof(led_value.value));
//     vTaskDelay(pdMS_TO_TICKS(5));
//   }
// }

extern "C" int main(void) {
	rcc_clock_setup_in_hse_8mhz_out_72mhz(); // For "blue pill"
	rcc_periph_clock_enable(RCC_GPIOC);

  // for spi pins
  // rcc_periph_clock_enable(RCC_GPIOA);

  // built-in LED
	gpio_set_mode(GPIOC,
                GPIO_MODE_OUTPUT_2_MHZ,
                GPIO_CNF_OUTPUT_PUSHPULL,
                GPIO13);

  spi_setup();
  spi_dma_setup();

  pwm_setup(TIM_OCM_PWM2);

	// xTaskCreate(task1,"LED",100,NULL,configMAX_PRIORITIES-1,NULL);

  // turn off LEDs 2, 3, 4
  timer_set_oc_value(TIM1, TIM_OC2, 0xFFFF);
  timer_set_oc_value(TIM1, TIM_OC3, 0xFFFF);
  timer_set_oc_value(TIM1, TIM_OC4, 0xFFFF);

  // blocks here, doesn't seem to receive anything
  // led_value.value = spi_read(SPI1);

  // first
  xTaskCreate(fade_task,"FADE1", 100, (void *) &led_fade_values[0], configMAX_PRIORITIES-1, NULL);

  xTaskCreate(set_led_value_task, "SET_LED_VALUE", 100, (void *)&led_value, configMAX_PRIORITIES-1, NULL);

  // this works, but instead of fading in and out, it's kind of flickering
  // when I changed the delay to 0 from 50, it flickers differently
  // xTaskCreate(read_spi_task, "READ_SPI_TASK", 100, NULL, configMAX_PRIORITIES-1, NULL);

  // xTaskCreate(read_spi_dma_task, "READ_SPI_DMA_TASK", 100, NULL, configMAX_PRIORITIES-1, NULL);
  read_from_spi_dma(&led_value.value, sizeof(led_value.value));

  // second
  // xTaskCreate(fade_task,"FADE2", 100, (void *) &led_fade_values[1], configMAX_PRIORITIES-1, NULL);

  // // third, aka small
  // xTaskCreate(fade_task,"FADE3", 100, (void *) &led_fade_values[2], configMAX_PRIORITIES-1, NULL);

  // // fourth
  // xTaskCreate(fade_task,"FADE4", 100, (void *) &led_fade_values[3], configMAX_PRIORITIES-1, NULL);

	vTaskStartScheduler();

	while(true) {
    gpio_toggle(GPIOC,GPIO13);
  }

	return 0;
}
