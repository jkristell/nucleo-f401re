#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_semihosting;
extern crate nucleo_f401re as board;

use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m_rt::entry;

use board::gpio::{Edge, ExtiPin};
use board::hal::prelude::*;
use board::hal::{interrupt, stm32};
use board::Interrupt;

use cortex_m::peripheral::Peripherals;
use stm32f4xx_hal::i2c::I2c;

use tpa2016::Tpa2016;

#[entry]
fn main() -> ! {
    // The Stm32 peripherals
    let mut device = stm32::Peripherals::take().unwrap();
    // The Cortex-m peripherals
    let mut core = Peripherals::take().unwrap();

    // Enable the clock for the SYSCFG
    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    // Constrain clock registers
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpiob = device.GPIOB.split();
    let scl = gpiob.pb8.into_alternate_af4();
    let sda = gpiob.pb9.into_alternate_af4();

    let i2c = I2c::i2c1(device.I2C1,
                        (scl, sda),
                        40.khz(),
                        clocks);

    let mut tpa = Tpa2016::new(i2c);

    tpa.gain(40).unwrap();

    loop {
    }
}


