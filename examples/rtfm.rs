#![no_main]
#![no_std]

extern crate panic_semihosting;
extern crate nucleo_f401re as board;

use cortex_m_semihosting::hprintln;
use rtfm::app;
use board::hal::stm32;
use board::hal::gpio::{
    Input, Output, PushPull, PullDown,
    gpioa::PA5, gpioc::PC13
};
use board::prelude::*;
use board::gpio::{Edge, ExtiPin};

#[app(device = board::hal::stm32)]
const APP: () = {

    // Late resources
    static mut EXTI: stm32::EXTI = ();
    static mut BUTTON: PC13<Input<PullDown>> = ();
    static mut LED: PA5<Output<PushPull>> = ();

    #[init]
    fn init() {
        // Cortex-M peripherals
        let _core: rtfm::Peripherals = core;

        // Device specific peripherals
        let mut device: stm32::Peripherals = device;

        // Configure PC13 (User Button) as an input
        let gpioc = device.GPIOC.split();
        let mut button = gpioc.pc13.into_pull_down_input();

        // Configure the led pin as an output
        let gpioa = device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Enable the clock for the SYSCFG
        device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

        // Enable interrupt on PC13
        button.make_interrupt_source(&mut device.SYSCFG);
        button.enable_interrupt(&mut device.EXTI);
        button.trigger_on_edge(&mut device.EXTI, Edge::RISING);

        // Setup the system clock
        let rcc = device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        hprintln!("init done").unwrap();

        EXTI = device.EXTI;
        LED = led;
        BUTTON = button;
    }

    #[idle]
    fn idle() -> ! {
        hprintln!("idle").unwrap();

        // The idle loop
        loop {}
    }

    #[interrupt(resources = [EXTI, LED, BUTTON])]
    fn EXTI15_10() {
        // Clear the interrupt
        resources.BUTTON.clear_interrupt_pending_bit(resources.EXTI);
        // Toggle the led
        resources.LED.toggle();
    }
};
