#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate nucleo_f401re as board;
extern crate panic_semihosting;

use cortex_m_rt::entry;

use board::hal::delay::Delay;
use board::hal::prelude::*;
use board::hal::stm32;
use board::spi::{self, Spi};
use cortex_m::peripheral::Peripherals;

use board::gpio::{Edge, ExtiPin};
use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use segment_display::SegmentDisplay;

use board::hal::interrupt;
use board::Interrupt;

static SIGNAL: AtomicUsize = ATOMIC_USIZE_INIT;

#[entry]
fn main() -> ! {
    // The Stm32 peripherals
    let mut device = stm32::Peripherals::take().unwrap();
    // The Cortex-m peripherals
    let mut core = Peripherals::take().unwrap();

    // Configure PC5 (User B1) as an input
    let gpioc = device.GPIOC.split();
    let mut button = gpioc.pc13.into_pull_down_input();

    // Enable the clock for the SYSCFG
    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    // Constrain clock registers
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let mut delay = Delay::new(core.SYST, clocks);

    // Enable external interrupt on PC13
    button.make_interrupt_source(&mut device.SYSCFG);
    button.enable_interrupt(&mut device.EXTI);
    button.trigger_on_edge(&mut device.EXTI, Edge::FALLING);

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

#[interrupt]
fn EXTI15_10() {
    // Clear the interrupt
    unsafe {
        (*stm32::EXTI::ptr()).pr.modify(|_, w| w.pr13().set_bit());
    }
    // Signal to the man loop that it should toggle the led.
    SIGNAL.fetch_add(1, Ordering::Relaxed);
}
