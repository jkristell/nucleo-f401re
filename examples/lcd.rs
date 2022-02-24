#![no_main]
#![no_std]

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;

use hd44780_driver::HD44780;
use nucleo_f401re::{hal::prelude::*, pac};

#[entry]
fn main() -> ! {
    let device = pac::Peripherals::take().unwrap();
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
    let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();

    // Get delay provider
    let mut delay = core.SYST.delay(&clocks);

    // Setup the driver
    let mut lcd = HD44780::new_4bit(rs, en, d4, d5, d6, d7, &mut delay).unwrap();

    lcd.reset(&mut delay).unwrap();
    lcd.clear(&mut delay).unwrap();
    let _ = lcd.write_str("Hello, World!", &mut delay).unwrap();
    lcd.set_cursor_pos(40, &mut delay).unwrap();
    let _ = lcd.write_str("Nucleo f401RE", &mut delay).unwrap();

    loop {
        continue;
    }
}
