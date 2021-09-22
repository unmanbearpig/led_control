#include <libopencm3/stm32/timer.h>

#define INITIAL_LED_VALUE 0xEEEE
#define LED_COUNT 3
/* Needed so we can check the current values from somewhere else.
 * Should be only set in `set_3_leds` */
static uint16_t led_values[LED_COUNT] = {
  INITIAL_LED_VALUE,
  INITIAL_LED_VALUE,
  INITIAL_LED_VALUE };

static bool led_mode[LED_COUNT] = {
  false, false, true,
};

const uint16_t BASE_PWM_PERIOD = 5126;
static uint16_t pwm_period = BASE_PWM_PERIOD;

void set_pwm_period(uint16_t new_pwm_period) {
  pwm_period = new_pwm_period;
  timer_set_period(TIM1, pwm_period);
}

void pwm_setup() {
  // https://github.com/ksarkies/ARM-Ports/blob/master/test-libopencm3-stm32f1/pwm-tim1.c
  /* already enabled */
  rcc_periph_clock_enable(RCC_GPIOA);

  gpio_set_mode(GPIOA, GPIO_MODE_OUTPUT_50_MHZ,
                GPIO_CNF_OUTPUT_ALTFN_PUSHPULL,
                GPIO8 | GPIO9 | GPIO10 /* | GPIO11 */);

  rcc_periph_clock_enable(RCC_TIM1);
  // timer_reset(TIM1); // missing

  timer_set_mode(TIM1, TIM_CR1_CKD_CK_INT, TIM_CR1_CMS_EDGE, TIM_CR1_DIR_UP);
  timer_set_prescaler(TIM1, 0); // it's the default, set it anyway to be extra certain

  /* Set Timer output compare mode:
   * - Channel 1
   * - PWM mode 2 (output low when CNT < CCR1, high otherwise)
   */
  /* temporary(?) hack to try to use different mode for one of the channels to 
   * try to reduce EMI from PWM */
  // timer_set_oc_mode(TIM1, TIM_OC1, TIM_OCM_PWM1);
  // timer_set_oc_mode(TIM1, TIM_OC2, oc_mode);
  // timer_set_oc_mode(TIM1, TIM_OC3, oc_mode);
  /* timer_set_oc_mode(TIM1, TIM_OC4, oc_mode); */

  set_led_mode(0, led_mode[0]);
  set_led_mode(1, led_mode[1]);
  set_led_mode(2, led_mode[2]);

  // this works without TIM_OC1N
  timer_enable_oc_output(TIM1, TIM_OC1);
  // this doesn't work without TIM_OC1, so does nothing?
  /* timer_enable_oc_output(TIM1, TIM_OC1N); /\* TODO: we don't need it do we? *\/ */

  timer_enable_oc_output(TIM1, TIM_OC2);
  /* timer_enable_oc_output(TIM1, TIM_OC2N); /\* TODO: we don't need it do we? *\/ */
  timer_enable_oc_output(TIM1, TIM_OC3);
  /* timer_enable_oc_output(TIM1, TIM_OC4); */

  // @note It is necessary to call this function to enable the output on an advanced
  // timer <b>even if break or deadtime features are not being used</b>.
  timer_enable_break_main_output(TIM1);

  /* Set the polarity of OCN to be high to match that of the OC, for switching
     the low MOSFET through an inverting level shifter */
  timer_set_oc_polarity_high(TIM1, TIM_OC2N);

  /* The ARR (auto-preload register) sets the PWM period to 62.5kHz from the
     72 MHz clock.*/
  timer_enable_preload(TIM1);
  timer_set_period(TIM1, pwm_period);

  /* The CCR1 (capture/compare register 1) sets the PWM duty cycle to default 50% */
  timer_enable_oc_preload(TIM1, TIM_OC1);
  timer_enable_oc_preload(TIM1, TIM_OC2);
  timer_enable_oc_preload(TIM1, TIM_OC3);
  /* timer_enable_oc_preload(TIM1, TIM_OC4); */

  /* Force an update to load the shadow registers */
  timer_generate_event(TIM1, TIM_EGR_UG);

  /* Start the Counter. */
  timer_enable_counter(TIM1);
}

enum tim_oc_id led_oc_id(uint8_t led_id) {
  enum tim_oc_id oc_id = TIM_OC1;
  switch(led_id) {
    case 0: oc_id = TIM_OC3; break;
    case 1: oc_id = TIM_OC2; break;
    case 2: oc_id = TIM_OC1; break;
  }
  return oc_id;
}

/* sets LED PWM mode to inverted or not inverted, true for inverted
 * led_id is the index in led_mode and led_values */
void set_led_mode(uint8_t led_id, bool is_inverted) {
  if (led_id >= 3) { return; }
  led_mode[led_id] = is_inverted;

  enum tim_oc_mode oc_mode;
  if (is_inverted) {
    oc_mode = TIM_OCM_PWM2;
  } else {
    oc_mode = TIM_OCM_PWM1;
  }

  enum tim_oc_id oc_id = led_oc_id(led_id);
  if (is_inverted) {
    timer_set_oc_mode(TIM1, oc_id, oc_mode);
  } else {
    timer_set_oc_mode(TIM1, oc_id, oc_mode);
  }
}

void update_leds_mode(bool *new_modes) {
  for (int i = 0; i < LED_COUNT; i++) {
    if (led_mode[i] != new_modes[i]) {
      set_led_mode(i, new_modes[i]);
    }
  }
}

void set_led_value(uint8_t led_id, uint16_t value) {
  if (led_id >= 3) { return; }

  led_values[led_id] = value;
  enum tim_oc_id oc_id = led_oc_id(led_id);

  uint16_t oc_value = value;
  if (led_mode[led_id]) {
    // is inverted
    oc_value = pwm_period - value;
  }

  timer_set_oc_value(TIM1, oc_id, oc_value);
}



void set_3_leds(uint16_t *leds) {
  // uint16_t pwm_period = 16383;
  set_led_value(0, leds[0]);
  set_led_value(1, leds[1]);
  set_led_value(2, leds[2]);

}
