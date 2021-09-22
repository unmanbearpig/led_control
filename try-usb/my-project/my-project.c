#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <errno.h>
#include "libopencm3/stm32/rcc.h"
#include "libopencm3/stm32/gpio.h"
#include "libopencm3/usb/usbd.h"
#include "libopencm3/usb/cdc.h"
#include <libopencm3/cm3/scb.h>
#include <libopencm3/stm32/usart.h>

#include "libopencm3/cm3/nvic.h"

#include "pwm.h"


#define LED1PORT GPIOC
#define LED1PIN GPIO13

#define DEBUG

// called by fwrite / printf
int _write(int file, char *ptr, int len);

static const struct usb_device_descriptor dev = {
  .bLength = USB_DT_DEVICE_SIZE,
  .bDescriptorType = USB_DT_DEVICE,
  .bcdUSB = 0x0200,
  .bDeviceClass = USB_CLASS_CDC,
  .bDeviceSubClass = 0,
  .bDeviceProtocol = 0,
  .bMaxPacketSize0 = 64,
  .idVendor = 0xCAFE,
  .idProduct = 0xCAFE,
  .bcdDevice = 0x0200,
  .iManufacturer = 1,
  .iProduct = 2,
  .iSerialNumber = 3,
  .bNumConfigurations = 1,
};

/*
 * This notification endpoint isn't implemented. According to CDC spec its
 * optional, but its absence causes a NULL pointer dereference in Linux
 * cdc_acm driver.
 */
static const struct usb_endpoint_descriptor comm_endp[] = {{
    .bLength = USB_DT_ENDPOINT_SIZE,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x83,
    .bmAttributes = USB_ENDPOINT_ATTR_INTERRUPT,
    .wMaxPacketSize = 16,
    .bInterval = 255,
  }};

static const struct usb_endpoint_descriptor data_endp[] = {{
    .bLength = USB_DT_ENDPOINT_SIZE,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x01,
    .bmAttributes = USB_ENDPOINT_ATTR_BULK,
    .wMaxPacketSize = 64,
    .bInterval = 1,
  }, {
    .bLength = USB_DT_ENDPOINT_SIZE,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x82,
    .bmAttributes = USB_ENDPOINT_ATTR_BULK,
    .wMaxPacketSize = 64,
    .bInterval = 1,
  }};

static const struct usb_endpoint_descriptor my_endp[] = {{
    .bLength = USB_DT_ENDPOINT_SIZE,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x05,
    .bmAttributes = USB_ENDPOINT_ATTR_INTERRUPT,
    .wMaxPacketSize = 64,
    .bInterval = 1,
  }, {
    .bLength = USB_DT_ENDPOINT_SIZE,
    .bDescriptorType = USB_DT_ENDPOINT,
    .bEndpointAddress = 0x86,
    .bmAttributes = USB_ENDPOINT_ATTR_INTERRUPT,
    .wMaxPacketSize = 64,
    .bInterval = 1,
  }};


static const struct {
  struct usb_cdc_header_descriptor header;
  struct usb_cdc_call_management_descriptor call_mgmt;
  struct usb_cdc_acm_descriptor acm;
  struct usb_cdc_union_descriptor cdc_union;
} __attribute__((packed)) cdcacm_functional_descriptors = {
  .header = {
    .bFunctionLength = sizeof(struct usb_cdc_header_descriptor),
    .bDescriptorType = CS_INTERFACE,
    .bDescriptorSubtype = USB_CDC_TYPE_HEADER,
    .bcdCDC = 0x0110,
  },
  .call_mgmt = {
    .bFunctionLength =
    sizeof(struct usb_cdc_call_management_descriptor),
    .bDescriptorType = CS_INTERFACE,
    .bDescriptorSubtype = USB_CDC_TYPE_CALL_MANAGEMENT,
    .bmCapabilities = 0,
    .bDataInterface = 1,
  },
  .acm = {
    .bFunctionLength = sizeof(struct usb_cdc_acm_descriptor),
    .bDescriptorType = CS_INTERFACE,
    .bDescriptorSubtype = USB_CDC_TYPE_ACM,
    .bmCapabilities = 0,
  },
  .cdc_union = {
    .bFunctionLength = sizeof(struct usb_cdc_union_descriptor),
    .bDescriptorType = CS_INTERFACE,
    .bDescriptorSubtype = USB_CDC_TYPE_UNION,
    .bControlInterface = 0,
    .bSubordinateInterface0 = 1,
  },
};

static const struct usb_interface_descriptor comm_iface[] = {{
    .bLength = USB_DT_INTERFACE_SIZE,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 0,
    .bAlternateSetting = 0,
    .bNumEndpoints = 1,
    .bInterfaceClass = USB_CLASS_CDC,
    .bInterfaceSubClass = USB_CDC_SUBCLASS_ACM,
    .bInterfaceProtocol = USB_CDC_PROTOCOL_AT,
    .iInterface = 0,

    .endpoint = comm_endp,

    .extra = &cdcacm_functional_descriptors,
    .extralen = sizeof(cdcacm_functional_descriptors),
  }};

static const struct usb_interface_descriptor data_iface[] = {{
    .bLength = USB_DT_INTERFACE_SIZE,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 1,
    .bAlternateSetting = 0,
    .bNumEndpoints = 2,
    .bInterfaceClass = USB_CLASS_DATA,
    .bInterfaceSubClass = 0,
    .bInterfaceProtocol = 0,
    .iInterface = 0,

    .endpoint = data_endp,
  }};

static const struct usb_interface_descriptor my_iface[] = {{
    .bLength = USB_DT_INTERFACE_SIZE,
    .bDescriptorType = USB_DT_INTERFACE,
    .bInterfaceNumber = 2,
    .bAlternateSetting = 0,
    .bNumEndpoints = 2,
    .bInterfaceClass = USB_CLASS_VENDOR,
    .bInterfaceSubClass = 0,
    .bInterfaceProtocol = 0,
    .iInterface = 0,

    .endpoint = my_endp,
  }};

static const struct usb_interface ifaces[] = {{
    .num_altsetting = 1,
    .altsetting = comm_iface,
  }, {
    .num_altsetting = 1,
    .altsetting = data_iface,
  }, {
    .num_altsetting = 1,
    .altsetting = my_iface,
  }};

static const struct usb_config_descriptor config = {
  .bLength = USB_DT_CONFIGURATION_SIZE,
  .bDescriptorType = USB_DT_CONFIGURATION,
  .wTotalLength = 0,
  .bNumInterfaces = 3,
  .bConfigurationValue = 1,
  .iConfiguration = 0,
  .bmAttributes = 0x80,
  .bMaxPower = 0x32,

  .interface = ifaces,
};

static const char *usb_strings[] = {
  "unmbp",
  "Hello kitty!",
  "DEMO"
};

/* Buffer to be used for control requests. */
uint8_t usbd_control_buffer[128];

static enum usbd_request_return_codes
cdcacm_control_request(usbd_device *usbd_dev,
                       struct usb_setup_data *req,
                       uint8_t **buf,
                       uint16_t *len,
                       void (**complete)(usbd_device *usbd_dev, 
                               struct usb_setup_data *req)) {
  (void)complete;
  (void)buf;
  (void)usbd_dev;

  switch (req->bRequest) {
  case USB_CDC_REQ_SET_CONTROL_LINE_STATE: {
    printf("-> USB_CDC_REQ_SET_CONTROL_LINE_STATE\r\n");
    /*
     * This Linux cdc_acm driver requires this to be implemented
     * even though it's optional in the CDC spec, and we don't
     * advertise it in the ACM functional descriptor.
     */
    char local_buf[10];
    struct usb_cdc_notification *notif = (void *)local_buf;

    /* We echo signals back to host as notification. */
    notif->bmRequestType = 0xA1;
    notif->bNotification = USB_CDC_NOTIFY_SERIAL_STATE;
    notif->wValue = 0;
    notif->wIndex = 0;
    notif->wLength = 2;
    local_buf[8] = req->wValue & 3;
    local_buf[9] = 0;
    // usbd_ep_write_packet(0x83, buf, 10);
    return USBD_REQ_HANDLED;
  }
  case USB_CDC_REQ_SET_LINE_CODING:
    printf("-> USB_CDC_REQ_SET_LINE_CODING\r\n");
    if (*len < sizeof(struct usb_cdc_line_coding))
      return USBD_REQ_NOTSUPP;
    return USBD_REQ_HANDLED;
  }

  printf("-> request %d => USBD_REQ_NOTSUPP\r\n", req->bRequest);
  return USBD_REQ_NOTSUPP;
}

static void cdcacm_data_rx_cb(usbd_device *usbd_dev, uint8_t ep)
{
  printf("-> cdcacm_data_rx_cb\r\n");
  (void)ep;
  (void)usbd_dev;

  char buf[64];
  int len = usbd_ep_read_packet(usbd_dev, 0x01, buf, 64);

  if (len) {
    if (len >= 4) {
      if (0 == memcmp(buf, "test", 4)) {
        gpio_toggle(LED1PORT, LED1PIN);

        char test_str[] = "Hello there!";
        usbd_ep_write_packet(usbd_dev, 0x82, test_str, sizeof(test_str)-1);
        buf[len] = 0;
      }

      if (len >= 5 && 0 == memcmp(buf, "reset", 5)) {
        char reset_str[] = "ReSeTting!!!";
        usbd_ep_write_packet(usbd_dev, 0x82, reset_str, sizeof(reset_str)-1);
        scb_reset_system();
      }
    } else {
      if (len > 2) {
        buf[0] = '@';
      }

      usbd_ep_write_packet(usbd_dev, 0x82, buf, len);
    }
    buf[len] = 0;
  }
}



static void custom_int_rx_cb(usbd_device *usbd_dev, uint8_t ep) {
  /* gpio_toggle(LED1PORT, LED1PIN); */
#ifdef DEBUG
  printf("-> custom_int_rx_cb\r\n");
#endif

  // TODO: how to read from interrupt endpiont?

  char buf[128];
  int len = usbd_ep_read_packet(usbd_dev, 0x05, buf, sizeof(buf));
#ifdef DEBUG
  printf("   got %d bytes\r\n", len);
  /* _write(1, buf, len); */
  printf("\r\n");
#endif

  if (len != 2 * 3) {
    printf("Wrong size packet: %d instead of %d\r\n", len, 2*3);
    return;
  }

  uint16_t *values = buf;
#ifdef DEBUG
  printf("bytes: %02x %02x %02x\r\n", values[0], values[1], values[2]);
#endif

  set_3_leds(values);

  // debugging
  /* char test_str[] = "Hello there!"; */
  /* usbd_ep_write_packet(usbd_dev, 0x82, test_str, sizeof(test_str)-1); */
  /* usbd_ep_write_packet(usbd_dev, 0x05, test_str, sizeof(test_str)-1); */
  /* usbd_ep_write_packet(usbd_dev, 0x86, test_str, sizeof(test_str)-1); */
}

static void custom_int_tx_cb(usbd_device *usbd_dev, uint8_t ep) {
  gpio_toggle(LED1PORT, LED1PIN);

  printf("-> custom_int_tx_cb\r\n");

  /* // char buf[64]; */
  /* int len = usbd_ep_read_packet(usbd_dev, 0x86, buf, 64); */
  /* // debugging */
  char test_str[] = "Hello from custom_int_tx_cb!\n";
  /* usbd_ep_write_packet(usbd_dev, 0x82, test_str, sizeof(test_str)-1); */
  /* usbd_ep_write_packet(usbd_dev, 0x05, test_str, sizeof(test_str)-1); */
  usbd_ep_write_packet(usbd_dev, 0x86, test_str, sizeof(test_str)-1);
}

static void cdcacm_set_config(usbd_device *usbd_dev, uint16_t wValue)
{
  printf("-> cdcacm_set_config\r\n");
  (void)wValue;
  (void)usbd_dev;

  usbd_ep_setup(usbd_dev, 0x01, USB_ENDPOINT_ATTR_BULK, 64, cdcacm_data_rx_cb);
  usbd_ep_setup(usbd_dev, 0x82, USB_ENDPOINT_ATTR_BULK, 64, NULL);
  usbd_ep_setup(usbd_dev, 0x83, USB_ENDPOINT_ATTR_INTERRUPT, 16, NULL);

  usbd_ep_setup(usbd_dev, 0x05, USB_ENDPOINT_ATTR_INTERRUPT, 64, custom_int_rx_cb);
  usbd_ep_setup(usbd_dev, 0x86, USB_ENDPOINT_ATTR_INTERRUPT, 64, custom_int_tx_cb);

  usbd_register_control_callback(usbd_dev,
                                 USB_REQ_TYPE_CLASS | USB_REQ_TYPE_INTERFACE,
                                 USB_REQ_TYPE_TYPE | USB_REQ_TYPE_RECIPIENT,
                                 cdcacm_control_request);
}

void usart_setup() {
  /* can't get USART2 to work */
  rcc_periph_clock_enable(RCC_TIM2);
  rcc_periph_clock_enable(RCC_AFIO);
  rcc_periph_clock_enable(RCC_USART2);
  gpio_set_mode(GPIOA, GPIO_MODE_OUTPUT_50_MHZ,
                GPIO_CNF_OUTPUT_ALTFN_PUSHPULL, GPIO_USART2_TX);
  usart_set_baudrate(USART2, 115200);
  usart_set_databits(USART2, 8);
  usart_set_stopbits(USART2, USART_STOPBITS_1);
  usart_set_mode(USART2, USART_MODE_TX);
  usart_set_parity(USART2, USART_PARITY_NONE);
  usart_set_flow_control(USART2, USART_FLOWCONTROL_NONE);

  usart_enable(USART2);
}

int _write(int file, char *ptr, int len)
{
  int i;

  if (file == 1) {
    for (i = 0; i < len; i++) {
      usart_send_blocking(USART2, ptr[i]);
    }
    return i;
  }

  errno = EIO;
  return -1;
}

void reconnect_usb() {
  printf("Pulling D+ down...\r\n");
  /* gpio_set_mode(GPIOC, GPIO_MODE_OUTPUT_2_MHZ, GPIO_CNF_OUTPUT_PUSHPULL, GPIO11); */

  /* Hack stolen from libopencm3/tests/gadget-zero/main-stm32f103-generic.c */
  /*
   * Vile hack to reenumerate, physically _drag_ d+ low.
   * (need at least 2.5us to trigger usb disconnect)
   */
  gpio_set_mode(GPIOA, GPIO_MODE_OUTPUT_2_MHZ,
                GPIO_CNF_OUTPUT_PUSHPULL, GPIO12);
  gpio_clear(GPIOA, GPIO12);

  for (int i = 0; i < 800000; i++) {
    __asm__("nop");
  }
}

uint32_t xorshift32(uint32_t *rnd) {
  uint32_t x = *rnd;
  x ^= x << 13;
  x ^= x >> 17;
  x ^= x << 5;
  *rnd = x;
  return x;
}

uint16_t lowest_non_zero_led_value() {
  uint16_t lowest = 0;
  for (int i = 0; i < LED_COUNT; i++) {
    uint16_t val = led_values[i];
    if (val == 0) { continue; }
    if (lowest == 0 || val < lowest) {
      lowest = val;
    }
  }

  return lowest;
}

/* returns true if duty cycle is high enough to make the LEDs produce 
 * significant amounts of EMI */
bool is_noisy() {
  const uint16_t noisy_floor = 6500;
  const uint16_t noisy_ceiling = 21500;
  // Noisy when:
  // - 7k 0 0
  // - 7k 7k 0
  //
  // Not noisy when:
  // - 7k 7k 15k (15k inverted)
  // - above 21k

  for (int i = 0; i < LED_COUNT; i++) {
    uint16_t val = led_values[i];
    if (val > noisy_floor && val < noisy_ceiling) {
      return true;
    }
  }
  return false;
}

// uint32_t rnd = 0x1CE4E5B9;

uint32_t tim_iter = 0;
/* Wiggle the frequency a bit so the EMI from PWM is a bit less noticeable */
void tim2_isr(void) {
  if (!is_noisy()) {
    goto done;
  }
  goto done;
  tim_iter += 1;
  int i = tim_iter % 17;

  /* - tim_iter % 128 doesn't do much */
  uint16_t new_period = BASE_PWM_PERIOD - (tim_iter % 128) * 8;
  /* sounds better than xorshift */
  switch(i) {
    case 0: new_period -= 0; break;
    case 1: new_period -= 731; break;
    case 2: new_period -= 111; break;
    case 3: new_period -= 1678; break;
    case 4: new_period -= 210; break;
    case 5: new_period -= 921; break;
    case 6: new_period -= 393; break;
    case 7: new_period -= 1457; break;
    case 8: new_period -= 151; break;
    case 9: new_period -= 917; break;
    case 10: new_period -= 478; break;
    case 11: new_period -= 691; break;
    case 12: new_period -= 1874; break;
    case 13: new_period -= 41; break;
    case 14: new_period -= 8; break;
    case 15: new_period -= 1141; break;
    case 16: new_period -= 71; break;
  }
  // rnd = xorshift32(&rnd);
  // uint16_t new_period = pwm_period - (rnd & 0x7ff);
  set_pwm_period(new_period);
done:
  timer_clear_flag(TIM2, TIM_SR_CC1IF);
}

int main(void)
{
  int i;

  usbd_device *usbd_dev;

  rcc_clock_setup_in_hse_8mhz_out_72mhz();
  rcc_periph_clock_enable(RCC_GPIOA); /* USART2 & USB & PWM */
  rcc_periph_clock_enable(RCC_GPIOC);

  gpio_set_mode(LED1PORT, GPIO_MODE_OUTPUT_50_MHZ, GPIO_CNF_OUTPUT_PUSHPULL,
      LED1PIN);
  usart_setup();
  printf("USART initialized!\r\n");

  printf("Setting up PWM...\r\n");
  pwm_setup();

  // Setup TIM2 for frequency wiggle
  rcc_periph_clock_enable(RCC_TIM2);
  nvic_enable_irq(NVIC_TIM2_IRQ);
  rcc_periph_reset_pulse(RST_TIM2);
  timer_set_mode(TIM2, TIM_CR1_CKD_CK_INT, TIM_CR1_CMS_EDGE, TIM_CR1_DIR_UP);
  timer_set_prescaler(TIM2, rcc_apb1_frequency * 2);
  timer_disable_preload(TIM2);
  timer_continuous_mode(TIM2);
  timer_set_period(TIM2, 2);
  timer_set_oc_value(TIM2, TIM_OC1, 1);
  timer_enable_counter(TIM2);
  timer_enable_irq(TIM2, TIM_DIER_CC1IE);

  uint16_t init_values[3] = { 0xFFFF, 0xAAAA, 0x1111 };
  set_3_leds(init_values);

  printf("Initializing USB...\r\n");
  usbd_dev = usbd_init(&st_usbfs_v1_usb_driver, &dev, &config, usb_strings, 3,
                  usbd_control_buffer, sizeof(usbd_control_buffer));
  usbd_register_set_config_callback(usbd_dev, cdcacm_set_config);

  for (i = 0; i < 0x800000; i++)
    __asm__("nop");
  gpio_clear(GPIOC, GPIO11);

  char test_str[] = "I'm here\r\n";
  uint32_t ipoll = 0;
  while (1) {
    ipoll++;
    usbd_poll(usbd_dev);

    if (ipoll % 400000 == 0) {
      gpio_toggle(LED1PORT, LED1PIN);

      printf("%s\r\n", test_str);
      usbd_ep_write_packet(usbd_dev, 0x82, test_str, sizeof(test_str)-1);
      usbd_ep_write_packet(usbd_dev, 0x86, test_str, sizeof(test_str)-1);
    }
  }
}
