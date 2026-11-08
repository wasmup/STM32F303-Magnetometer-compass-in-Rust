//! STM32F3DISCOVERY GPIO, button and LED operations.
use crate::register::{modify, read, write};

const RCC_AHBENR: usize = 0x4002_1014;
const SYST_CSR: usize = 0xe000_e010;
const SYST_RVR: usize = 0xe000_e014;
const SYST_CVR: usize = 0xe000_e018;
const GPIOA: usize = 0x4800_0000;
const GPIOB: usize = 0x4800_0400;
const GPIOE: usize = 0x4800_1000;
const USER_BUTTON: u32 = 1 << 0;
const SCL: u32 = 1 << 6;
const SDA: u32 = 1 << 7;
const LED_PINS: [u32; 8] = [13, 14, 15, 8, 9, 10, 11, 12];

pub fn init() {
    modify(RCC_AHBENR, 0, (1 << 17) | (1 << 18) | (1 << 21));
    // The blue user button is PA0, active high with an external pull-down.
    modify(GPIOA, 3, 0);
    modify(GPIOA + 0x0c, 3, 0);
    modify(GPIOE, 0xffff_0000, 0x5555_0000);
    leds(0);
    write(GPIOB + 0x18, SCL | SDA);
    modify(GPIOB + 0x04, 0, SCL | SDA);
    modify(GPIOB + 0x0c, (3 << 12) | (3 << 14), (1 << 12) | (1 << 14));
    modify(GPIOB, (3 << 12) | (3 << 14), (1 << 12) | (1 << 14));
}

pub fn user_button_pressed() -> bool {
    read(GPIOA + 0x10) & USER_BUTTON != 0
}

pub fn leds(mask: u8) {
    let mut set = 0;
    for (i, pin) in LED_PINS.iter().enumerate() {
        if mask & (1 << i) != 0 {
            set |= 1 << pin;
        }
    }
    write(GPIOE + 0x18, set | ((0xff00 & !set) << 16));
}
fn delay_100ms() {
    // SysTick runs from the reset-default 8 MHz HSI system clock.
    write(SYST_RVR, 800_000 - 1);
    write(SYST_CVR, 0);
    write(SYST_CSR, 0x5); // Processor clock, counter enabled, no interrupt.
    while read(SYST_CSR) & (1 << 16) == 0 {}
    write(SYST_CSR, 0);
}
pub fn led_test() {
    leds(0);
    for bit in 0..8 {
        leds(1 << bit);
        delay_100ms();
        leds(0);
    }
}
pub fn calibration_confirmation() {
    leds(0xff);
    delay_100ms();
    leds(0);
}
