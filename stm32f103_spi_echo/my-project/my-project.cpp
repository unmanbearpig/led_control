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

extern "C" void vApplicationStackOverflowHook( TaskHandle_t xTask, char *pcTaskName );

void vApplicationStackOverflowHook( TaskHandle_t xTask __attribute((unused)), char *pcTaskName __attribute((unused))) {
	for(;;);	// Loop forever here..
}

typedef struct {
  uint32_t isrs;
  uint32_t tcif_count;
  uint32_t htif_count;
  uint32_t teif_count;

} DmaChanStats;

DmaChanStats chan2_stats = { 0 };
DmaChanStats chan3_stats = { 0 };

#define BUF_SIZE 16

char buf[BUF_SIZE];

char dummy_buf[BUF_SIZE];

void dma1_channel2_isr() {
  chan2_stats.isrs += 1;

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_TCIF)) {
    chan2_stats.tcif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_TCIF);
  }

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_HTIF)) {
    chan2_stats.htif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_HTIF);
  }

  if (dma_get_interrupt_flag(DMA1, DMA_CHANNEL2, DMA_TEIF)) {
    chan2_stats.teif_count += 1;
    dma_clear_interrupt_flags(DMA1, DMA_CHANNEL2, DMA_TEIF);
  }
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
}

void spi_echo() {
  // SPI setup
  rcc_periph_clock_enable(RCC_SPI1);

  gpio_set_mode(GPIOA,
                GPIO_MODE_OUTPUT_50_MHZ,
                GPIO_CNF_OUTPUT_ALTFN_PUSHPULL, // ALTFN or no?
                GPIO6);

  gpio_set_mode(GPIOA, GPIO_MODE_OUTPUT_50_MHZ, GPIO_CNF_OUTPUT_ALTFN_PUSHPULL, GPIO6);

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
  spi_set_dff_8bit(SPI1); // seems like it's not needed

  // need this? probably
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
  // spi_enable_rx_buffer_not_empty_interrupt(SPI1);
  // spi_enable_error_interrupt(SPI1);

  spi_enable(SPI1);

  // DMA setup

  rcc_periph_clock_enable(RCC_DMA1);

  // not sure what it does and where it should be
  dma_channel_reset(DMA1, DMA_CHANNEL2);
  dma_channel_reset(DMA1, DMA_CHANNEL3);

  dma_set_peripheral_address(DMA1, DMA_CHANNEL2, (uint32_t)&SPI1_DR);
  dma_set_peripheral_address(DMA1, DMA_CHANNEL3, (uint32_t)&SPI1_DR);

 	nvic_set_priority(NVIC_DMA1_CHANNEL2_IRQ, 0);
  nvic_set_priority(NVIC_DMA1_CHANNEL3_IRQ, 0);

	nvic_enable_irq(NVIC_DMA1_CHANNEL2_IRQ);
  nvic_enable_irq(NVIC_DMA1_CHANNEL3_IRQ);

  dma_disable_peripheral_increment_mode(DMA1, DMA_CHANNEL2);
  dma_disable_peripheral_increment_mode(DMA1, DMA_CHANNEL3);

  dma_set_memory_address(DMA1, DMA_CHANNEL2, (uint32_t)buf);
  dma_set_memory_address(DMA1, DMA_CHANNEL3, (uint32_t)dummy_buf);

  dma_set_number_of_data(DMA1, DMA_CHANNEL2, sizeof(buf));
  dma_set_number_of_data(DMA1, DMA_CHANNEL3, sizeof(buf));

  dma_set_memory_size(DMA1, DMA_CHANNEL2, DMA_CCR_MSIZE_8BIT);
  dma_set_memory_size(DMA1, DMA_CHANNEL3, DMA_CCR_MSIZE_8BIT);

  dma_set_peripheral_size(DMA1, DMA_CHANNEL2, DMA_CCR_PSIZE_8BIT);
  dma_set_peripheral_size(DMA1, DMA_CHANNEL3, DMA_CCR_PSIZE_8BIT);

  dma_set_priority(DMA1, DMA_CHANNEL2, DMA_CCR_PL_LOW);
  dma_set_priority(DMA1, DMA_CHANNEL3, DMA_CCR_PL_LOW);

  dma_set_read_from_peripheral(DMA1, DMA_CHANNEL2);
  dma_set_read_from_memory(DMA1, DMA_CHANNEL3);

  dma_enable_memory_increment_mode(DMA1, DMA_CHANNEL2);
  dma_enable_memory_increment_mode(DMA1, DMA_CHANNEL3);

  dma_enable_circular_mode(DMA1, DMA_CHANNEL2);
  // not sure about this one, probably no
  dma_enable_circular_mode(DMA1, DMA_CHANNEL3);

  dma_enable_channel(DMA1, DMA_CHANNEL2);
  dma_enable_channel(DMA1, DMA_CHANNEL3);

  spi_enable_rx_dma(SPI1);
  spi_enable_tx_dma(SPI1);

  dma_enable_transfer_complete_interrupt(DMA1, DMA_CHANNEL2);
  // dma_enable_half_transfer_interrupt(DMA1, DMA_CHANNEL2);
  // dma_enable_transfer_error_interrupt(DMA1, DMA_CHANNEL2);

  dma_enable_transfer_complete_interrupt(DMA1, DMA_CHANNEL3);
}

extern "C" int main(void) {
	rcc_clock_setup_in_hse_8mhz_out_72mhz(); // For "blue pill"
	rcc_periph_clock_enable(RCC_GPIOC);

  memset(buf, 0, BUF_SIZE);
  memset(dummy_buf, 0xBA, BUF_SIZE);

  // for spi pins, seems to be not needed?
  rcc_periph_clock_enable(RCC_GPIOA);

  // built-in LED
	gpio_set_mode(GPIOC,
                GPIO_MODE_OUTPUT_2_MHZ,
                GPIO_CNF_OUTPUT_PUSHPULL,
                GPIO13);

  // for debugging
  // gpio_set_mode(GPIOA, GPIO_MODE_OUTPUT_50_MHZ, GPIO_CNF_OUTPUT_PUSHPULL, GPIO2 | GPIO6 | GPIO8 | GPIO9 | GPIO10 | GPIO11);

  // while(true) {
  //   gpio_toggle(GPIOA, GPIO2 | GPIO6 | GPIO8 | GPIO9 | GPIO10 | GPIO11);
  // }

  spi_echo();

	while(true) {
    gpio_toggle(GPIOC,GPIO13);
  }

	return 0;
}
