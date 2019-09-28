#![no_main]
#![no_std]

use core::cell::RefCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;
use panic_semihosting as _;

use nucleo_f401re::{
    delay::Delay,
    gpio::{Edge, ExtiPin},
    hal::{
        gpio::{gpioc::PC13, Input, PullDown},
        interrupt,
    },
    prelude::*,
    spi::{self, Spi},
    stm32, Interrupt, EXTI,
};

use segment_display::SegmentDisplay;

static SIGNAL: AtomicUsize = AtomicUsize::new(0);

static BUTTON: Mutex<RefCell<Option<PC13<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static EXTI: Mutex<RefCell<Option<EXTI>>> = Mutex::new(RefCell::new(None));


#[entry]
fn main() -> ! {
    // The Stm32 peripherals
    let mut device = stm32::Peripherals::take().unwrap();
    // The Cortex-m peripherals
    let core = Peripherals::take().unwrap();

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

    let exti = device.EXTI;
    cortex_m::interrupt::free(|cs| {
        EXTI.borrow(cs).replace(Some(exti));
        BUTTON.borrow(cs).replace(Some(button));
    });


    // Enable the external interrupt
    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI15_10);
    }

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
    cortex_m::interrupt::free(|cs| {
        let mut button = BUTTON.borrow(cs).borrow_mut();
        let mut exti = EXTI.borrow(cs).borrow_mut();
        let mut extiref = exti.as_mut().unwrap();
        button
            .as_mut()
            .unwrap()
            .clear_interrupt_pending_bit(&mut extiref);
    });
    // Signal to the man loop that it should toggle the led.
    SIGNAL.fetch_add(1, Ordering::Relaxed);
}
