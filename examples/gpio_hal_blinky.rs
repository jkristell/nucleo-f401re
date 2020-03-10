#![no_main]
#![no_std]

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;
use panic_semihosting as _;

use nucleo_f401re::{delay::Delay, prelude::*, stm32};

#[entry]
fn main() -> ! {
    let p = stm32::Peripherals::take().unwrap();
    let cp = Peripherals::take().unwrap();

    let gpioa = p.GPIOA.split();

    // (Re-)configure PA5 (LD2 - User Led) as output
    let mut led = gpioa.pa5.into_push_pull_output();
    led.set_low().ok();

    // Constrain clock registers
    let rcc = p.RCC.constrain();

    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    // Get delay provider
    let mut delay = Delay::new(cp.SYST, clocks);

    loop {
        delay.delay_ms(500_u16);
        led.toggle().ok();
    }
}
