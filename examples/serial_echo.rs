#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_rtt_target as _;

use nucleo_f401re::{
    hal::{
        prelude::*,
        serial::{config::Config, Serial},
    },
    pac,
};

use nb::block;

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let device = pac::Peripherals::take().unwrap();

    let gpioa = device.GPIOA.split();
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let tx = gpioa.pa2.into_alternate_af7();
    let rx = gpioa.pa3.into_alternate_af7();

    let config = Config::default().baudrate(115_200.bps());

    let serial = Serial::usart2(device.USART2, (tx, rx), config, clocks).unwrap();

    let (mut tx, mut rx) = serial.split();

    loop {
        // Read character and echo it back
        let received = block!(rx.read()).unwrap();
        block!(tx.write(received)).ok();
    }
}
