#![no_main]
#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};

use panic_semihosting as _;
use cortex_m_rt::entry;
use cortex_m::peripheral::Peripherals;

use nucleo_f401re::{
    delay::Delay,
    prelude::*,
    stm32,
    timer::Timer,
    Interrupt,
    spi::{self, Spi},
};

use segment_display::SegmentDisplay;


static SIGNAL: AtomicUsize = AtomicUsize::new(0);

#[entry]
fn main() -> ! {
    let device = stm32::Peripherals::take().unwrap();
    let mut core = Peripherals::take().unwrap();

    // Enable the clock for the SYSCFG
    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    // Constrain clock registers
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let mut delay = Delay::new(core.SYST, clocks);

    let timer2 = Timer::tim2(device.TIM2,
                             10.khz(),
                             clocks);

    //timer2.

    // Enable the external interrupt
    core.NVIC.enable(Interrupt::EXTI15_10);

    let gpiob = device.GPIOB.split();
    let sck = gpiob.pb3.into_alternate_af5();
    //let miso = gpiob.pb4.into_alternate_af5();
    let miso = spi::NoMiso;
    let mosi = gpiob.pb5.into_alternate_af5();

    // rclk
    let latch = gpiob.pb4.into_push_pull_output();

    let mode = spi::Mode {
        polarity: spi::Polarity::IdleHigh,
        phase: spi::Phase::CaptureOnFirstTransition,
    };

    device.SPI1.cr2.modify(|_r, w| w.txdmaen().enabled());


    let spi = Spi::spi1(
        device.SPI1,
        (sck, miso, mosi),
        mode,
        10_000_000.hz(),
        clocks,
    );

    let mut segment_display = SegmentDisplay::new(spi, latch);

    loop {
        let v = SIGNAL.load(Ordering::Relaxed);
        segment_display.write_number(v);
        segment_display.refresh().unwrap();

        delay.delay_us(200_u16);
    }
}

