#![no_main]
#![no_std]

#![allow(deprecated)]

use core::cell::RefCell;

use cortex_m::peripheral::Peripherals;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use panic_semihosting as _;

use nucleo_f401re::{
    hal::interrupt,
    Interrupt,
    prelude::*,
    stm32::{self, TIM2},
    timer::{Timer, Event},
};
use stm32f4xx_hal::gpio::{gpioa::PA5, PushPull, Output};

static TIMER: Mutex<RefCell<Option<Timer<TIM2>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<PA5<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let p = stm32::Peripherals::take().unwrap();
    let mut core = Peripherals::take().unwrap();

    // Enable the clock for the SYSCFG
    p.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    let gpioa = p.GPIOA.split();

    // (Re-)configure PA5 (LD2 - User Led) as output
    let led = gpioa.pa5.into_push_pull_output();

    cortex_m::interrupt::free(|cs| {
        LED.borrow(cs).replace(Some(led));
    });

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    // Setup timer
    let mut timer = Timer::tim2(p.TIM2, 1.hz(), clocks);

    // Enable interrupt
    timer.listen(Event::TimeOut);

    cortex_m::interrupt::free(|cs| {
        TIMER.borrow(cs).replace(Some(timer));
    });

    // Enable the external interrupt
    core.NVIC.enable(Interrupt::TIM2);

    loop {
    }
}

#[interrupt]
fn TIM2() {
    // Ack the interrupt
    unsafe {
        (*stm32::TIM2::ptr()).sr.modify(|_, w| w.uif().clear_bit());
    }

    cortex_m::interrupt::free(|cs| {
        let mut led = LED.borrow(cs).borrow_mut();
        led
            .as_mut()
            .unwrap()
            .toggle();
    });
}

