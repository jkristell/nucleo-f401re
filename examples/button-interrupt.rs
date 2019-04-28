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

static SIGNAL: AtomicBool = AtomicBool::new(false);

#[entry]
fn main() -> ! {
    // The Stm32 peripherals
    let mut device = stm32::Peripherals::take().unwrap();
    // The Cortex-m peripherals
    let mut core = Peripherals::take().unwrap();

    // Configure PA5 (LD2 - User led) as an output
    let gpioa = device.GPIOA.split();
    let mut led = gpioa.pa5.into_push_pull_output();

    // Configure PC5 (User B1) as an input
    let gpioc = device.GPIOC.split();
    let mut button = gpioc.pc13.into_pull_up_input();

    // Enable the clock for the SYSCFG
    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    // Constrain clock registers
    let rcc = device.RCC.constrain();
    let _clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    // Enable external interrupt on PC13
    button.make_interrupt_source(&mut device.SYSCFG);
    button.enable_interrupt(&mut device.EXTI);
    button.trigger_on_edge(&mut device.EXTI, Edge::RISING);

    // Enable the external interrupt
    core.NVIC.enable(Interrupt::EXTI15_10);

    loop {
        let state_change = SIGNAL.load(Ordering::Relaxed);
        if state_change {
            led.toggle();
            SIGNAL.store(false, Ordering::Relaxed);
        }
    }
}


#[interrupt]
fn EXTI15_10() {
    // Clear the interrupt
    unsafe {
        (*stm32::EXTI::ptr()).pr.modify(|_, w| { w.pr13().set_bit() });
    }
    // Signal to the man loop that it should toggle the led.
    SIGNAL.store(true, Ordering::Relaxed);
}
