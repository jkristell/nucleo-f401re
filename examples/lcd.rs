#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_semihosting;

extern crate nucleo_f401re as board;
extern crate hd44780_driver;

use core::fmt::Write;

use cortex_m_rt::entry;

use board::hal::delay::Delay;
use board::hal::prelude::*;
use board::hal::stm32;

use cortex_m::peripheral::Peripherals;
use hd44780_driver::HD44780;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let gpioa = p.GPIOA.split();
        let gpiob = p.GPIOB.split();
        let gpioc = p.GPIOC.split();

        // Configure the pins as outputs
        let mut d7 = gpioa.pa0.into_push_pull_output();
        let mut d6 = gpioa.pa1.into_push_pull_output();
        let mut d5 = gpioa.pa4.into_push_pull_output();
        let mut d4 = gpiob.pb0.into_push_pull_output();
        let mut rs = gpioc.pc1.into_push_pull_output();
        let mut en = gpioc.pc0.into_push_pull_output();

        // Constrain clock registers
        let mut rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        // Setup the driver
        let mut lcd = HD44780::new_4bit(rs, en, d4, d5, d6, d7, delay);

        lcd.reset();
        lcd.clear();
        let _ = lcd.write_str("Hello, World!");
        lcd.set_cursor_pos(40);
        let _ = lcd.write_str("Nucleo f401RE");
    }

    loop {}
}
