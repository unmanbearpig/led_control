
#define PWM_PERIOD 0xFFFF

void pwm_setup(enum tim_oc_mode oc_mode) {
  // https://github.com/ksarkies/ARM-Ports/blob/master/test-libopencm3-stm32f1/pwm-tim1.c
  rcc_periph_clock_enable(RCC_GPIOA);

  gpio_set_mode(GPIOA, GPIO_MODE_OUTPUT_50_MHZ,
                GPIO_CNF_OUTPUT_ALTFN_PUSHPULL,
                GPIO8 | GPIO9 | GPIO10 | GPIO11);

  // rcc_set_ppre2(0); // nah
  rcc_periph_clock_enable(RCC_TIM1);
  // timer_reset(TIM1); // missing

  /* Set Timer global mode:
   * - No division
   * - Alignment centre mode 1 (up/down counting, interrupt on downcount only)
   * - Direction up (when centre mode is set it is read only, changes by hardware)
   */
  timer_set_mode(TIM1, TIM_CR1_CKD_CK_INT, TIM_CR1_CMS_CENTER_1, TIM_CR1_DIR_UP);

  timer_set_prescaler(TIM1, 0);

  /* Set Timer output compare mode:
   * - Channel 1
   * - PWM mode 2 (output low when CNT < CCR1, high otherwise)
   */
	timer_set_oc_mode(TIM1, TIM_OC1, oc_mode);
	timer_set_oc_mode(TIM1, TIM_OC2, oc_mode);


  // maybe we need other channels or something?
  timer_set_oc_mode(TIM1, TIM_OC3, oc_mode);
  timer_set_oc_mode(TIM1, TIM_OC4, oc_mode);


  // this works without TIM_OC1N
	timer_enable_oc_output(TIM1, TIM_OC1);
  // this doesn't work without TIM_OC1, so does nothing?
	timer_enable_oc_output(TIM1, TIM_OC1N);

	timer_set_oc_mode(TIM1, TIM_OC2, TIM_OCM_PWM2);

  timer_enable_oc_output(TIM1, TIM_OC2);
	timer_enable_oc_output(TIM1, TIM_OC2N);

  // try more channels
  timer_enable_oc_output(TIM1, TIM_OC3);
	// timer_enable_oc_output(TIM1, TIM_OC3N);
  timer_enable_oc_output(TIM1, TIM_OC4);
	// timer_enable_oc_output(TIM1, TIM_OC4N); // no such thing as OC4N

  // @note It is necessary to call this function to enable the output on an advanced
  // timer <b>even if break or deadtime features are not being used</b>.
  timer_enable_break_main_output(TIM1);


  /* Set the polarity of OCN to be high to match that of the OC, for switching
     the low MOSFET through an inverting level shifter */
  timer_set_oc_polarity_high(TIM1, TIM_OC2N);

  /* The ARR (auto-preload register) sets the PWM period to 62.5kHz from the
     72 MHz clock.*/
	timer_enable_preload(TIM1);
	timer_set_period(TIM1, PWM_PERIOD);


  /* The CCR1 (capture/compare register 1) sets the PWM duty cycle to default 50% */
	timer_enable_oc_preload(TIM1, TIM_OC1);
	// timer_set_oc_value(TIM1, TIM_OC1, 10000);
	timer_enable_oc_preload(TIM1, TIM_OC2);
	// timer_set_oc_value(TIM1, TIM_OC2, 20000);

  // try more channels
	timer_enable_oc_preload(TIM1, TIM_OC3);
	// timer_set_oc_value(TIM1, TIM_OC3, 30000);
  timer_enable_oc_preload(TIM1, TIM_OC4);
	// timer_set_oc_value(TIM1, TIM_OC4, 40000);



  /* Force an update to load the shadow registers */
	timer_generate_event(TIM1, TIM_EGR_UG);

  /* Start the Counter. */
	timer_enable_counter(TIM1);
}

void set_all_leds(uint16_t led1, uint16_t led2, uint16_t led3, uint16_t led4) {
  timer_set_oc_value(TIM1, TIM_OC1, (0xFFFF-led1));
  timer_set_oc_value(TIM1, TIM_OC2, (0xFFFF-led2));
  timer_set_oc_value(TIM1, TIM_OC3, (0xFFFF-led3));
  timer_set_oc_value(TIM1, TIM_OC4, (0xFFFF-led4));
}
