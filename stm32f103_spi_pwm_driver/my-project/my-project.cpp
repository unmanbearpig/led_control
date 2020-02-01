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
#include "../../common_headers/structs.h"

#include "pwm.h"

extern "C" void vApplicationStackOverflowHook( TaskHandle_t xTask, char *pcTaskName );

void vApplicationStackOverflowHook( TaskHandle_t xTask __attribute((unused)), char *pcTaskName __attribute((unused))) {
	for(;;);	// Loop forever here..
}

uint32_t led_delay_ms = 1500;

LedValuesMessage tmp_msg =
  {
   .magic = 0,
   .led1_value = 0,
   .led2_value = 0,
   .led3_value = 0,
   .led4_value = 0,
  };


LedValuesMessage msg =
  {
   .magic = led_values_message_magic,
   .led1_value = 0,
   .led2_value = 0,
   .led3_value = 0,
   .led4_value = 0,
  };

struct LedFadeValue {
  tim_oc_id channel; // e.g. TIM_OC2
  uint16_t value;
  uint16_t min_value;
  uint16_t max_value;
  int16_t step;
};

struct LedValue {
  tim_oc_id channel; // e.g. TIM_OC2
  uint16_t value;
};

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
  spi_disable_ss_output(SPI1);

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

void read_from_spi_dma(void *rx_buf, uint32_t rx_len) {
  uint32_t dma = DMA1;
  uint8_t channel = DMA_CHANNEL2;

  dma_set_memory_address(dma, channel, (uint32_t)rx_buf);
  // Linux doesn't support 16 bits?
  dma_set_memory_size(dma, channel, DMA_CCR_MSIZE_8BIT);
  dma_set_peripheral_size(dma, channel, DMA_CCR_PSIZE_8BIT);
  dma_set_number_of_data(dma, channel, rx_len);
  dma_set_priority(dma, channel, DMA_CCR_PL_LOW);
  dma_set_read_from_peripheral(dma, channel);

  dma_enable_memory_increment_mode(dma, channel);
  dma_enable_channel(dma, channel);
  dma_enable_circular_mode(dma, channel);

  spi_enable_rx_dma(SPI1);

  dma_enable_transfer_complete_interrupt(dma, channel);
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

struct LedValue tmp_led_value =
  { TIM_OC3, 0x0030 };

void start_dma() {
  read_from_spi_dma(&tmp_msg, sizeof(tmp_msg));
}

void stop_dma() {
  spi_disable_rx_dma(SPI1);
  dma_disable_channel(DMA1, DMA_CHANNEL2);
  spi_clean_disable(SPI1);
  spi_disable(SPI1);
}

void restart_dma() {
  stop_dma();
  spi_setup();
  spi_dma_setup();
  start_dma();
}

static void set_msg_values_task(void *_value) {
  LedValuesMessage *msg = (LedValuesMessage *)_value;

  // we don't need it do we?
  if (!is_msg_valid(msg)) {
    restart_dma();
    set_msg_to_error_state(msg);
    return;
  }

  for(;;) {
    set_all_leds(msg->led1_value, msg->led2_value, msg->led3_value, msg->led4_value);
    vTaskDelay(pdMS_TO_TICKS(10));
  }
}

void dma1_channel2_isr() {
  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_TCIF)) {
    if (is_msg_valid(&tmp_msg)) {
      memcpy(&msg, &tmp_msg, sizeof(tmp_msg));
    } else {
      // We (the slave) might go out of sync with the host,
      // i.e. we incorrectly assume the start of the message
      // it might happen if the slave was (re)started in the middle of communication
      // Here we try to resynchronize with the host by just restarting SPI DMA
      // Eventually we should catch the correct start of the message and stop receiving errors
      // There probably is a better way, but it works for now
      restart_dma();
      // Debugging, so I notice the errors better.
      // Should change it no not changing state
      set_msg_to_error_state(&msg);
    }

    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_TCIF);
  } else {
    restart_dma();
  }
}

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

  xTaskCreate(set_msg_values_task, "SET_MSG_LED_VALUE", 100, (void *)&msg, configMAX_PRIORITIES-1, NULL);

  start_dma();

	vTaskStartScheduler();

	while(true) {
    gpio_toggle(GPIOC,GPIO13);
  }

	return 0;
}
