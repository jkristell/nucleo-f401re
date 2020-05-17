#![no_main]
#![no_std]
#![allow(deprecated)]

use rtt_target::{rprintln, rtt_init_print};
use panic_rtt_target as _;

use nucleo_f401re::{
    Led, Button,
    gpio::Edge,
    prelude::*,
};

use rtfm::app;

#[app(device = nucleo_f401re::hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        button: Button,
        led: Led,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        rtt_init_print!();
        // Device specific peripherals
        let mut device = ctx.device;

        // Enable the clock for the SYSCFG
        device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());
        // Setup the system clock
        let rcc = device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        let gpioa = device.GPIOA.split();
        let gpioc = device.GPIOC.split();

        // Setup Button and enable external interrupt
        let mut button = Button::new(gpioc.pc13);
        button.enable_interrupt(Edge::RISING, &mut device.SYSCFG, &mut device.EXTI);

        // Setup the led
        let led = Led::new(gpioa.pa5);

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
