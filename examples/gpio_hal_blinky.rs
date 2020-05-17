#![no_main]
#![no_std]

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target;

use nucleo_f401re::{
    Led,
    hal::{
        delay::Delay,
        prelude::*,
    },
    pac
};

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let p = pac::Peripherals::take().unwrap();
    let cp = Peripherals::take().unwrap();

    let gpioa = p.GPIOA.split();

    // (Re-)configure PA5 (LD2 - User Led) as output
    let mut led = Led::new(gpioa.pa5);
    led.set(false);

    // Constrain clock registers
    let rcc = p.RCC.constrain();

    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    // Get delay provider
    let mut delay = Delay::new(cp.SYST, clocks);

    loop {
        delay.delay_ms(500_u16);
        led.toggle();
    }
}
