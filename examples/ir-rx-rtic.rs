#![no_main]
#![no_std]

use defmt_rtt as _;
use dwt_systick_monotonic::DwtSystick;
use panic_probe as _;

use infrared::{
    protocol::Nec,
    receiver::{Event, PinInput},
    ConstReceiver,
};
use nucleo_f401re::{
    hal::{
        gpio::gpioa::PA10,
        gpio::{Edge, Floating, Input},
        prelude::*,
    },
    Led,
};
use rtic::rtic_monotonic::Instant;

#[rtic::app(device = nucleo_f401re::pac, peripherals = true)]
mod app {
    use super::*;

    const CORE_CLOCK: u32 = 84_000_000;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<84_000_000>;
    type IrProto = Nec;
    type IrReceivePin = PA10<Input<Floating>>;
    type IrReceiver = ConstReceiver<Nec, Event, PinInput<IrReceivePin>, CORE_CLOCK>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Led,
        recv: crate::app::IrReceiver,
        last_event: Instant<MyMono>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Device specific peripherals
        let mut device = ctx.device;

        // Setup the system clock
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();
        let mut syscfg = device.SYSCFG.constrain();
        let gpioa = device.GPIOA.split();

        let mono_clock = clocks.hclk().0;

        let monot = DwtSystick::new(&mut ctx.core.DCB, ctx.core.DWT, ctx.core.SYST, mono_clock);

        defmt::debug!("Mono clock: {}", mono_clock);

        // Setup the board led
        let led = Led::new(gpioa.pa5);

        // Setup the infrared receiver
        let recv = {
            let mut ir_pin = gpioa.pa10;
            ir_pin.make_interrupt_source(&mut syscfg);
            ir_pin.enable_interrupt(&mut device.EXTI);
            ir_pin.trigger_on_edge(&mut device.EXTI, Edge::RisingFalling);

            infrared::Receiver::builder()
                .protocol::<IrProto>()
                .event_driven()
                .pin(ir_pin)
                .build_const::<{ crate::app::CORE_CLOCK }>()
        };

        defmt::debug!("init done");
        (
            Shared {},
            Local {
                led,
                recv,
                last_event: Instant::new(0),
            },
            init::Monotonics(monot),
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {}
    }

    #[task(binds = EXTI15_10, local = [last_event, led, recv])]
    fn on_pin_irq(ctx: on_pin_irq::Context) {
        let led = ctx.local.led;
        let recv = ctx.local.recv;
        let last_event = ctx.local.last_event;

        let now = monotonics::MyMono::now();
        let dt = now.checked_duration_since(last_event).unwrap().integer();

        match recv.event(dt) {
            Ok(Some(cmd)) => {
                defmt::info!("CMD: {:?}", cmd);
            }
            Ok(None) => {}
            Err(err) => defmt::error!("Recv error: {:?}", err),
        }

        // Update Timestamp
        *last_event = now;

        // Clear pin interrupt
        recv.pin().clear_interrupt_pending_bit();

        // Toggle the led
        led.toggle();
    }
}
