#include <FreeRTOS/FreeRTOS.h>
#include <FreeRTOS/task.h>

#include <string.h>
#include <math.h>
#include <libopencm3/stm32/rcc.h>
#include <libopencm3/stm32/gpio.h>
#include <libopencm3/cm3/scb.h>
#include <libopencm3/stm32/exti.h>
#include <libopencm3/cm3/nvic.h>
#include <libopencm3/stm32/f1/nvic.h>
#include <libopencm3/stm32/spi.h>
#include <libopencm3/stm32/dma.h>
#include <libopencm3/cm3/cortex.h>
#include "../../common/protocol.h"

#include "../../common/stm32_pwm.h"

extern "C" void vApplicationStackOverflowHook( TaskHandle_t xTask, char *pcTaskName );

void vApplicationStackOverflowHook( TaskHandle_t xTask __attribute((unused)), char *pcTaskName __attribute((unused))) {
	for(;;);	// Loop forever here..
}

uint16_t pwm_period = 16383;

LedValuesMessage input_msg_buf;

LedValuesMessage output_msgs[2];
LedValuesMessage *output_msg;

#define INITIAL_LED_VALUE 0xEEEE
uint16_t led_values[LED_COUNT + 1] = { INITIAL_LED_VALUE, INITIAL_LED_VALUE, INITIAL_LED_VALUE, INITIAL_LED_VALUE, INITIAL_LED_VALUE };
float float_led_values[LED_COUNT] = { 0, 0, 0, 0 };
int use_float = 0;
float led_gamma = 2.0;

volatile int spi_command_received = 0;

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
  // dma_disable_peripheral_increment_mode(DMA1, DMA_CHANNEL3);
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

void write_to_spi_dma(volatile void *tx_buf, uint32_t tx_len) {
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

  // should be enabled
  dma_enable_circular_mode(dma, channel);

  spi_enable_tx_dma(SPI1);

  dma_enable_transfer_complete_interrupt(dma, channel);
  dma_enable_half_transfer_interrupt(dma, channel);
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
  read_from_spi_dma(&input_msg_buf, sizeof(input_msg_buf));
  write_to_spi_dma(output_msgs, sizeof(output_msgs));
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

void init_tmp_fade_down_to(uint16_t *led_values, uint16_t *value, uint16_t target_value) {
  for(; *value > target_value; *value = *value*=0.99 ) {
    if (spi_command_received) {
      break;
    }

    for(int i = 0; i < LED_COUNT; i++) {
      led_values[i] = *value;
    }

    vTaskDelay(pdMS_TO_TICKS(25));
  }
}

static void set_temp_initial_values_task(void *_value) {
  uint16_t *led_values = (uint16_t *)_value;
  vTaskDelay(pdMS_TO_TICKS(3000));

  uint16_t value = INITIAL_LED_VALUE;

  init_tmp_fade_down_to(led_values, &value, 0x8888);
  vTaskDelay(pdMS_TO_TICKS(5000));
  init_tmp_fade_down_to(led_values, &value, 0x0111);
  vTaskDelay(pdMS_TO_TICKS(2000));
  init_tmp_fade_down_to(led_values, &value, 0);
  vTaskDelete(NULL);
}

void led_values_convert_float_to_16(uint16_t *led_values16, float *led_values_float) {
  for(int i = 0; i < LED_COUNT; i++) {
    float val = powf(led_values_float[i], led_gamma) * pwm_period;
    if (val > pwm_period) {
      val = pwm_period;
    }

    led_values16[i] = val;
  }
}

static void set_msg_values_task(void *_value) {
  // uint16_t *led_values = (uint16_t *)_value;

  for(;;) {
    // if (use_float) {
    //   // xxx
    //   float temp = float_led_values[2];
    //   float_led_values[2] = float_led_values[3];
    //   float_led_values[3] = temp;

    //   // ----

    //   led_values_convert_float_to_16(led_values, float_led_values);
    // }

    set_4_leds(led_values, pwm_period);

    // vTaskDelay(pdMS_TO_TICKS(50));
    vTaskDelay(1); // xxx?
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

// not optimized at all
int is_all_zeros(void *buf, size_t len) {
  uint8_t *chars = (uint8_t *)buf;
  for (size_t i = 0; i < len; i++) {
    if (chars[i] != 0) {
      return 0;
    }
  }

  return 1;
}

void handle_values_msg(LedValuesMessage *input_msg, LedValuesMessage *output_msg) {
  for(int i = 0; i < LED_COUNT; i++) {
    output_msg->payload.data.values.values16[i] = led_values[i];
  }

  if (input_msg->type & LED_WRITE && input_msg->payload.data.flags & LED_VALUES_FLAG_FLOAT) {
    use_float = true;

    for(int i = 0; i < LED_COUNT; i++) {
      float input_value = input_msg->payload.data.values.values_float[i];
      float amount = input_msg->payload.data.amount;
      float old_value = float_led_values[i];

      float val = 0;
      if (input_msg->payload.data.flags & LED_VALUES_FLAG_ADD) {
        val = old_value + (amount * input_value);
      } else {
        val = (old_value * (1.0 - amount)) + (amount * input_value);
      }

      float_led_values[i] = val;
    }

    led_values_convert_float_to_16(led_values, float_led_values);
    set_4_leds(led_values, pwm_period);
  } else {
    // if (input_msg->type & LED_WRITE) {
    //   use_float = false;

    //   for(int i = 0; i < LED_COUNT; i++) {
    //     led_values[i] = input_msg->payload.data.values.values16[i];
    //   }

    //   set_4_leds(led_values);
    // }

    // output_msg->magic = LED_VALUES_MESSAGE_MAGIC;
    // output_msg->payload.data.values.values16[0] = led_values[0];
    // output_msg->payload.data.values.values16[1] = led_values[1];
    // output_msg->payload.data.values.values16[2] = led_values[2];
    // output_msg->payload.data.values.values16[3] = led_values[3];
  }
}

void handle_config_msg(LedValuesMessage *input_msg, LedValuesMessage *output_msg) {
  if (input_msg->type & LED_WRITE) {
    if (input_msg->payload.config.flags & LED_CONFIG_SET_GAMMA) {
      led_gamma = input_msg->payload.config.gamma;
    }

    if (input_msg->payload.config.flags & LED_CONFIG_SET_PWM_PERIOD) {
      uint16_t new_pwm_period = input_msg->payload.config.pwm_period;
      if (new_pwm_period != pwm_period) {
        pwm_period = new_pwm_period;
        set_pwm_period(pwm_period);
      }
    }

    led_values_convert_float_to_16(led_values, float_led_values);
    set_4_leds(led_values, pwm_period);
  }

  if (input_msg->type & LED_READ) {
    memset(output_msg, 0, sizeof(*output_msg));
    output_msg->type = LED_CONFIG;
    output_msg->payload.config.gamma = led_gamma;
    output_msg->payload.config.pwm_period = pwm_period;
    // TODO
  }
}

void handle_msg(LedValuesMessage *input_msg, LedValuesMessage *output_msg) {
  if (is_all_zeros(input_msg, sizeof(*input_msg))) {
    output_msg->magic = LED_VALUES_MESSAGE_MAGIC;
    output_msg->type = LED_READ | LED_WRITE;
    output_msg->payload.data.flags = 0;
    output_msg->payload.data.values.values16[0] = led_values[0];
    output_msg->payload.data.values.values16[1] = led_values[1];
    output_msg->payload.data.values.values16[2] = led_values[2];
    output_msg->payload.data.values.values16[3] = led_values[3];
  } else if (is_msg_valid(input_msg)) {
    if (input_msg->type & LED_CONFIG) {
      handle_config_msg(input_msg, output_msg);
    } else {
      handle_values_msg(input_msg, output_msg);
    }
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

    led_values[0] = 0;
    led_values[1] = pwm_period;
    led_values[2] = 0;
    led_values[3] = pwm_period;
  }
}

void dma1_channel2_isr() {
  cm_disable_interrupts();

  chan2_stats.isrs += 1;
  spi_command_received = 1;

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_TCIF)) {
    chan2_stats.tcif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_TCIF);

    handle_msg(&input_msg_buf, output_msg);
  }

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_HTIF)) {
    chan2_stats.htif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_HTIF);
  }

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_TEIF)) {
    chan2_stats.teif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_TEIF);
  }

  cm_enable_interrupts();
}


void dma1_channel3_isr() {
  cm_disable_interrupts();
  chan3_stats.isrs += 1;

  // complete
  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL3, DMA_TCIF)) {
    chan3_stats.tcif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL3, DMA_TCIF);

    output_msg = &output_msgs[1];
  }

  // half transfer
  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL3, DMA_HTIF)) {
    chan3_stats.htif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL3, DMA_HTIF);

    output_msg = &output_msgs[0];
  }

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL3, DMA_TEIF)) {
    chan3_stats.teif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL3, DMA_TEIF);
  }

  cm_enable_interrupts();
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

  memset((void *)&input_msg_buf, 0, sizeof(input_msg_buf));
  memset((void *)output_msgs, 0, sizeof(output_msgs));
  output_msg = &output_msgs[0];

  spi_setup();
  spi_dma_setup();
  start_dma();

  pwm_setup(TIM_OCM_PWM2, pwm_period);


  // xTaskCreate(set_msg_values_task, "SET_MSG_LED_VALUE", 100, (void *)&led_values, configMAX_PRIORITIES-1, NULL);
  // xTaskCreate(set_temp_initial_values_task, "SET_TEMP_INITAIL_VALUE", 100, (void *)&led_values, configMAX_PRIORITIES-1, NULL);

	// vTaskStartScheduler();

	while(true) {
    gpio_toggle(GPIOC,GPIO13);
  }

	return 0;
}
