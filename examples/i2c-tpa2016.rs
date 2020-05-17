#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::rprintln;

use nucleo_f401re::{i2c::I2c, prelude::*, stm32};

use tpa2016d2::{AgcPreset, Tpa2016d2};

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    // The Stm32 peripherals
    let device = stm32::Peripherals::take().unwrap();

    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpiob = device.GPIOB.split();
    let scl = gpiob
        .pb8
        .into_alternate_af4()
        .internal_pull_up(true)
        .set_open_drain();

    let sda = gpiob
        .pb9
        .into_alternate_af4()
        .internal_pull_up(true)
        .set_open_drain();

    let i2c = I2c::i2c1(device.I2C1, (scl, sda), 200.khz(), clocks);

    let mut tpa = Tpa2016d2::new(i2c);

    // Print the registers
    for i in 1..=7 {
        let v = tpa.device_reg(i).unwrap();
        rprintln!("{}: {}", i, v);
    }

    // Set the gain
    tpa.gain(32).unwrap();

    // Should print 32
    rprintln!("gain: {}", tpa.device_reg(5).unwrap());

    tpa.set_agc_preset(AgcPreset::Jazz).unwrap();

    loop {}
}
