#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[rtic::app(device = nucleo_f401re::pac, peripherals = true)]
mod app {

    use nucleo_f401re::{
        hal::{gpio::Edge, prelude::*},
        Button, Led,
    };

    #[shared]
    struct Resources {
        button: Button,
        led: Led,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Resources, Local, init::Monotonics) {
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
        button.enable_interrupt(Edge::Rising, &mut syscfg, &mut device.EXTI);

        // Setup the led
        let led = Led::new(gpioa.pa5);

        defmt::info!("init done");

        (Resources { led, button }, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        // The idle loop
        loop {}
    }

    #[task(binds = EXTI15_10, shared = [led, button])]
    fn on_button_press(ctx: on_button_press::Context) {
        let mut led = ctx.shared.led;
        let mut button = ctx.shared.button;

        // Clear the interrupt
        button.lock(|b| b.clear_interrupt_pending_bit());

        // Toggle the led
        led.lock(|l| l.toggle());

        defmt::info!("Button pressed!");
    }
}
