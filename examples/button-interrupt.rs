#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_semihosting;
extern crate nucleo_f401re as board;

use core::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};

use cortex_m_rt::entry;

use board::hal::prelude::*;
use board::hal::{interrupt, stm32};
use board::Interrupt;

use cortex_m::peripheral::Peripherals;

static LED_STATE: AtomicBool = ATOMIC_BOOL_INIT;

#[entry]
fn main() -> ! {
    // The Stm32 peripherals
    let device = stm32::Peripherals::take().unwrap();
    // The Cortex-m peripherals
    let mut core = Peripherals::take().unwrap();

    // Configure PA5 (LD2 - User led) as an output
    let gpioa = device.GPIOA.split();
    let mut led = gpioa.pa5.into_push_pull_output();

    // Configure PC5 (User B1) as an input
    let gpioc = device.GPIOC.split();
    let _button = gpioc.pc13.into_pull_up_input();

    // Enable the clock for the SYSCFG
    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    // Constrain clock registers
    let rcc = device.RCC.constrain();
    let _clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    // Enable interrupt on PC13
    device.SYSCFG.exticr4.write(|w| unsafe { w.exti13().bits(0b0010) });

    // Enable the interrupt on line 13
    device.EXTI.imr.modify(|_, w| w.mr13().set_bit());
    // Rising edge
    device.EXTI.rtsr.modify(|_, w| w.tr13().set_bit());
    // Falling edge
    //d.EXTI.ftsr.modify(|_, w| w.tr13().set_bit());

    // Enable the external interrupt
    core.NVIC.enable(Interrupt::EXTI15_10);

    loop {
        //let led_on = unsafe {LED_ON};
        let led_on = LED_STATE.load(Ordering::Relaxed);
        if led_on {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}


#[interrupt]
fn EXTI15_10() {
    // Clear the interrupt
    unsafe {
        (*stm32::EXTI::ptr()).pr.modify(|_, w| { w.pr13().set_bit() });
    }
    // Flip the bool
    LED_STATE.fetch_xor(true, Ordering::SeqCst);
}
