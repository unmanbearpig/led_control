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

#include "pwm.h"

extern "C" void vApplicationStackOverflowHook( TaskHandle_t xTask, char *pcTaskName );

void vApplicationStackOverflowHook( TaskHandle_t xTask __attribute((unused)), char *pcTaskName __attribute((unused))) {
	for(;;);	// Loop forever here..
}

uint32_t led_delay_ms = 1500;

LedValuesMessage input_msg =
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
  // need it?
  rcc_periph_clock_enable(RCC_GPIOA);

  rcc_periph_clock_enable(RCC_SPI1);

  gpio_set_mode(GPIOA,
                GPIO_MODE_OUTPUT_50_MHZ,
                GPIO_CNF_OUTPUT_ALTFN_PUSHPULL, // ALTFN or no?
                GPIO6);
  gpio_set_mode(GPIOA,
                GPIO_MODE_INPUT,
                GPIO_CNF_INPUT_FLOAT,
                GPIO4|GPIO5|GPIO7);
  // // ???
  // gpio_set_af(GPIOA, GPIO_AF5, GPIO5 | GPIO6 | GPIO7);

  // SPI1_I2SCFGR = 0;

  spi_disable(SPI1);
  spi_reset(SPI1);
  spi_disable_ss_output(SPI1);
  // spi_set_dff_8bit(SPI1); // seems like it's not needed

  // need this?
  spi_set_full_duplex_mode(SPI1);

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
  // spi_enable_rx_buffer_not_empty_interrupt(SPI1);
  // spi_enable_error_interrupt(SPI1);
}

void spi_dma_setup_output() {
  dma_disable_peripheral_increment_mode(DMA1, DMA_CHANNEL3);
  dma_channel_reset(DMA1, DMA_CHANNEL3);
  dma_set_peripheral_address(DMA1, DMA_CHANNEL3, (uint32_t)&SPI1_DR);

  nvic_set_priority(NVIC_DMA1_CHANNEL3_IRQ, 0);
  nvic_enable_irq(NVIC_DMA1_CHANNEL3_IRQ);
}


// https://www.rhye.org/post/stm32-with-opencm3-2-spi-and-dma/
void spi_dma_setup() {
  rcc_periph_clock_enable(RCC_DMA1);

  // not sure what it does and where it should be
  dma_channel_reset(DMA1, DMA_CHANNEL2);
  dma_set_peripheral_address(DMA1, DMA_CHANNEL2, (uint32_t)&SPI1_DR);

 	nvic_set_priority(NVIC_DMA1_CHANNEL2_IRQ, 0);
	nvic_enable_irq(NVIC_DMA1_CHANNEL2_IRQ);

  spi_dma_setup_output();
}

void write_to_spi_dma(void *tx_buf, uint32_t tx_len) {
  uint32_t dma = DMA1;
  uint8_t channel = DMA_CHANNEL3;

  dma_set_memory_address(dma, channel, (uint32_t)tx_buf);
  dma_set_memory_size(dma, channel, DMA_CCR_MSIZE_8BIT);
  dma_set_peripheral_size(dma, channel, DMA_CCR_PSIZE_8BIT);
  dma_set_number_of_data(dma, channel, tx_len);
  dma_set_priority(dma, channel, DMA_CCR_PL_LOW);
  dma_set_read_from_memory(dma, channel);
  dma_enable_memory_increment_mode(dma, channel);
  dma_enable_channel(dma, channel);

  // not sure about this one, probably no
  dma_enable_circular_mode(dma, channel);

  spi_enable_tx_dma(SPI1);

  // dma_enable_transfer_complete_interrupt(dma, channel);
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

void start_dma() {
  read_from_spi_dma(&input_msg, sizeof(input_msg));
  write_to_spi_dma(&msg, sizeof(msg));
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

typedef struct {
  uint32_t isrs;
  uint32_t tcif_count;
  uint32_t htif_count;
  uint32_t teif_count;

} DmaChanStats;

DmaChanStats chan2_stats = { 0 };
DmaChanStats chan3_stats = { 0 };

void dma1_channel2_isr() {
  chan2_stats.isrs += 1;

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_TCIF)) {
    chan2_stats.tcif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_TCIF);

    if (is_msg_valid(&input_msg)) {
      memcpy(&msg, &input_msg, sizeof(input_msg));
    } else if (is_msg_read_request(&input_msg)) {
      // if circular dma is enabled (now it is) it will return the thing anyway
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
  }

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_HTIF)) {
    chan2_stats.htif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_HTIF);
  }

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_TEIF)) {
    chan2_stats.teif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_TEIF);
  }


  // if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_TCIF)) {
  //   if (is_msg_valid(&input_msg)) {
  //     memcpy(&msg, &input_msg, sizeof(input_msg));
  //   } else {
  //     // We (the slave) might go out of sync with the host,
  //     // i.e. we incorrectly assume the start of the message
  //     // it might happen if the slave was (re)started in the middle of communication
  //     // Here we try to resynchronize with the host by just restarting SPI DMA
  //     // Eventually we should catch the correct start of the message and stop receiving errors
  //     // There probably is a better way, but it works for now
  //     restart_dma();
  //     // Debugging, so I notice the errors better.
  //     // Should change it no not changing state
  //     set_msg_to_error_state(&msg);
  //   }

  //   dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_TCIF);
  // } else {
  //   set_valid_msg_magic(&msg);
  //   msg.led1_value = 0;
  //   msg.led2_value = 0;
  //   msg.led3_value = 0;
  //   msg.led4_value = 0xFFFF;

  //   restart_dma();
  // }
}

void dma1_channel3_isr() {
  chan3_stats.isrs += 1;

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL3, DMA_TCIF)) {
    chan3_stats.tcif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL3, DMA_TCIF);
  }

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL3, DMA_HTIF)) {
    chan3_stats.htif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL3, DMA_HTIF);
  }

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL3, DMA_TEIF)) {
    chan3_stats.teif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL3, DMA_TEIF);
  }

  // spi_disable_tx_dma(SPI1);
  // write_to_spi_dma(&tx_msg, (sizeof(tx_msg)));

  // breaks the leds when I comment it out, but the isrs number goes up
  // dma_clear_interrupt_flags(DMA1, DMA_CHANNEL3, DMA_TCIF);
}

// static void write_to_spi_dma_task(void *_arg __attribute((unused))) {
//   for(;;) {
//     spi_disable_tx_dma(SPI1);

//     SPI1_DR = 0xFF;
//     vTaskDelay(pdMS_TO_TICKS(1));

//     write_to_spi_dma(&tx_msg, (sizeof(tx_msg)));
//   }
// }

extern "C" int main(void) {
	rcc_clock_setup_in_hse_8mhz_out_72mhz(); // For "blue pill"
	rcc_periph_clock_enable(RCC_GPIOC);

  rcc_periph_clock_enable(RCC_GPIOA);

  // built-in LED
	gpio_set_mode(GPIOC,
                GPIO_MODE_OUTPUT_2_MHZ,
                GPIO_CNF_OUTPUT_PUSHPULL,
                GPIO13);

  spi_setup();
  spi_dma_setup();
  start_dma();

  pwm_setup(TIM_OCM_PWM2);

  xTaskCreate(set_msg_values_task, "SET_MSG_LED_VALUE", 100, (void *)&msg, configMAX_PRIORITIES-1, NULL);
  // xTaskCreate(write_to_spi_dma_task, "START_DMA_TX_VALUE", 100, (void *)&msg, configMAX_PRIORITIES-1, NULL);


	vTaskStartScheduler();

	while(true) {
    gpio_toggle(GPIOC,GPIO13);
  }

	return 0;
}
