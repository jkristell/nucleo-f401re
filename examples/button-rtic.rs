#![no_main]
#![no_std]

#[rtic::app(device = nucleo_f401re::pac, peripherals = true)]
mod app {
    use panic_rtt_target as _;
    use rtt_target::{rprintln, rtt_init_print};

    use nucleo_f401re::{
        hal::{gpio::Edge, prelude::*},
        Button, Led,
    };

    #[resources]
    struct Resources {
        button: Button,
        led: Led,
    }

    #[init]
    fn init(ctx: init::Context) -> (init::LateResources, init::Monotonics) {
        rtt_init_print!();
        // Device specific peripherals
        let mut device = ctx.device;

        // Setup the system clock
        let rcc = device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        let mut syscfg = device.SYSCFG.constrain();

        let gpioa = device.GPIOA.split();
        let gpioc = device.GPIOC.split();

        // Setup Button and enable external interrupt
        let mut button = Button::new(gpioc.pc13);
        button.enable_interrupt(Edge::RISING, &mut syscfg, &mut device.EXTI);

        // Setup the led
        let led = Led::new(gpioa.pa5);

        rprintln!("init done");

        (init::LateResources { led, button }, init::Monotonics())
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        rprintln!("idle");

        // The idle loop
        loop {}
    }

    #[task(binds = EXTI15_10, resources = [led, button])]
    fn on_button_press(ctx: on_button_press::Context) {
        let on_button_press::Resources { mut led, mut button } = ctx.resources;

        // Clear the interrupt
        button.lock(|b| b.clear_interrupt_pending_bit());

        // Toggle the led
        led.lock(|l| l.toggle());

        rprintln!("Button pressed!");
    }
}
