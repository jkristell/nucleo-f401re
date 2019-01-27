#![no_main]
#![no_std]

extern crate cortex_m_rt;
extern crate panic_semihosting;
extern crate cortex_m_semihosting;
extern crate nucleo_f401re as board;
extern crate heapless;

#[macro_use(block)]
extern crate nb;

use board::hal::prelude::*;
use board::hal::stm32;

use board::hal::serial::{Serial, config::Config};

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use heapless::spsc::Queue; // fixed capacity `std::Vec`
use heapless::consts::U8; // type level integer used to specify capacity


static mut RB: Option<Queue<u8, U8>>  = None;

#[entry]
fn main() -> ! {

    let device = stm32::Peripherals::take().unwrap();

    let gpioa = device.GPIOA.split();
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let tx = gpioa.pa9.into_alternate_af7();
    let rx = gpioa.pa10.into_alternate_af7();

    let config = Config::default()
        .baudrate(31250.bps());

    let serial = Serial::usart1(device.USART1,
                                (tx, rx),
                                config, 
                                clocks).unwrap();

    let (_, mut rx) = serial.split();

    
    unsafe { RB = Some(Queue::new()) };
    // NOTE(unsafe) beware of aliasing the `consumer` end point
    let mut consumer = unsafe { RB.as_mut().unwrap().split().1 };

    hprintln!("Init done").unwrap();

    loop {
        // Read character and echo it back
        let r1 = block!(rx.read()).unwrap();
        let r2 = block!(rx.read()).unwrap();
        let r3 = block!(rx.read()).unwrap();
        let r4 = block!(rx.read()).unwrap();
        let r5 = block!(rx.read()).unwrap();
        hprintln!("{}", r1).unwrap();
        hprintln!("{}", r2).unwrap();
        hprintln!("{}", r3).unwrap();
        hprintln!("{}", r4).unwrap();
        hprintln!("{}", r5).unwrap();
    }
}
