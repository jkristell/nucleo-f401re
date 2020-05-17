#![no_main]
#![no_std]
#![allow(deprecated)]

use rtt_target::{rprintln, rtt_init_print};
use panic_rtt_target as _;

use nucleo_f401re::{
    Led,
    gpio::{gpioc::PC13, Edge, ExtiPin, Input, PullDown},
    prelude::*,
};

use rtfm::app;

#[app(device = nucleo_f401re::hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        button: PC13<Input<PullDown>>,
        led: Led,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        rtt_init_print!();
        // Device specific peripherals
        let mut device = ctx.device;

        // Configure PC13 (User Button) as an input
        let gpioc = device.GPIOC.split();
        let mut button = gpioc.pc13.into_pull_down_input();

        // Configure the led pin as an output
        let gpioa = device.GPIOA.split();
        let led = Led::new(gpioa.pa5);

        // Enable the clock for the SYSCFG
        device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

        // Enable interrupt on PC13
        button.make_interrupt_source(&mut device.SYSCFG);
        button.enable_interrupt(&mut device.EXTI);
        button.trigger_on_edge(&mut device.EXTI, Edge::RISING);

        // Setup the system clock
        let rcc = device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        rprintln!("init done");

        init::LateResources { led, button }
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        rprintln!("idle");

        // The idle loop
        loop {}
    }

    #[task(binds = EXTI15_10, resources = [led, button])]
    fn on_button_press(ctx: on_button_press::Context) {
        let on_button_press::Resources { led, button } = ctx.resources;

        // Clear the interrupt
        button.clear_interrupt_pending_bit();
        // Toggle the led
        led.toggle();
    }
};
