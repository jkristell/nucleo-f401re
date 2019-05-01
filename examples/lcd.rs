#![no_main]
#![no_std]

use core::fmt::Write;

use cortex_m_rt::entry;
use cortex_m::peripheral::Peripherals;
use panic_semihosting as _;

use nucleo_f401re::{
    hal::delay::Delay,
    hal::prelude::*,
    hal::stm32,
};

use hd44780_driver::HD44780;


#[entry]
fn main() -> ! {

    let device = stm32::Peripherals::take().unwrap();
    let core = Peripherals::take().unwrap();

    let gpioa = device.GPIOA.split();
    let gpiob = device.GPIOB.split();
    let gpioc = device.GPIOC.split();

    // Configure the pins as outputs
    let d7 = gpioa.pa0.into_push_pull_output();
    let d6 = gpioa.pa1.into_push_pull_output();
    let d5 = gpioa.pa4.into_push_pull_output();
    let d4 = gpiob.pb0.into_push_pull_output();
    let rs = gpioc.pc1.into_push_pull_output();
    let en = gpioc.pc0.into_push_pull_output();

    // Constrain clock registers
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    // Get delay provider
    let delay = Delay::new(core.SYST, clocks);

    // Setup the driver
    let mut lcd = HD44780::new_4bit(rs, en, d4, d5, d6, d7, delay);

    lcd.reset();
    lcd.clear();
    let _ = lcd.write_str("Hello, World!");
    lcd.set_cursor_pos(40);
    let _ = lcd.write_str("Nucleo f401RE");

    loop {

    }
}
