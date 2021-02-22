#![no_main]
#![no_std]

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;

use nucleo_f401re::{
    hal::{i2c::I2c, prelude::*},
    pac,
};

use tpa2016d2::{AgcPreset, Tpa2016d2};

#[entry]
fn main() -> ! {
    // The Stm32 peripherals
    let device = pac::Peripherals::take().unwrap();

    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpiob = device.GPIOB.split();
    let scl = gpiob
        .pb8
        .into_alternate_open_drain();

    let sda = gpiob
        .pb9
        .into_alternate_open_drain();

    let i2c = I2c::new(device.I2C1, (scl, sda), 200.khz(), clocks);

    let mut tpa = Tpa2016d2::new(i2c);

    // Print the registers
    for i in 1..=7 {
        let v = tpa.device_reg(i).unwrap();
        defmt::debug!("{}: {}", i, v);
    }

    // Set the gain
    tpa.gain(32).unwrap();

    // Should print 32
    defmt::debug!("gain: {}", tpa.device_reg(5).unwrap());

    tpa.set_agc_preset(AgcPreset::Jazz).unwrap();

    loop {}
}
