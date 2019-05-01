#![no_main]
#![no_std]

use cortex_m_rt::entry;
use cortex_m::peripheral::Peripherals;
use panic_semihosting as _;

use nucleo_f401re::{
    hal::delay::Delay,
    hal::prelude::*,
    hal::stm32,
};

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let gpioa = p.GPIOA.split();

        // (Re-)configure PA5 (LD2 - User Led) as output
        let mut led = gpioa.pa5.into_push_pull_output();
        led.set_low();

        // Constrain clock registers
        let rcc = p.RCC.constrain();

        let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        loop {
            // Turn LED on
            led.set_high();

            // Delay twice for half a second due to limited timer resolution
            delay.delay_ms(500_u16);
            delay.delay_ms(500_u16);

            // Turn LED off
            led.set_low();

            // Delay twice for half a second due to limited timer resolution
            delay.delay_ms(500_u16);
            delay.delay_ms(500_u16);
        }
    }

    loop {}
}
