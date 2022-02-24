#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[rtic::app(device = nucleo_f401re::pac, peripherals = true)]
mod app {
    use dwt_systick_monotonic::DwtSystick;

    use infrared::protocol::capture::Capture;
    use infrared::{
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

    const CORE_CLOCK: u32 = 84_000_000;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<CORE_CLOCK>;

    type IrProto = Capture;
    type IrReceivePin = PA10<Input<Floating>>;
    type IrReceiver = ConstReceiver<IrProto, Event, PinInput<IrReceivePin>, CORE_CLOCK>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Led,
        recv: crate::app::IrReceiver,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Device specific peripherals
        let mut device = ctx.device;

        // Setup the system clock
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();
        let mut syscfg = device.SYSCFG.constrain();
        let gpioa = device.GPIOA.split();

        let mono_clock = clocks.sysclk();

        let monot = DwtSystick::new(
            &mut ctx.core.DCB,
            ctx.core.DWT,
            ctx.core.SYST,
            clocks.hclk().to_Hz(),
        );

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
        (Shared {}, Local { led, recv }, init::Monotonics(monot))
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {}
    }

    #[task(binds = EXTI15_10, local = [prev: Option<dwt_systick_monotonic::fugit::TimerInstantU32<84_000_000> > = None, led, recv])]
    fn on_pin_irq(ctx: on_pin_irq::Context) {
        let led = ctx.local.led;
        let recv = ctx.local.recv;
        let prev = ctx.local.prev;

        let now = monotonics::MyMono::now();

        // Clear pin interrupt
        recv.pin().clear_interrupt_pending_bit();

        if let Some(last) = prev {
            //let dt = now.checked_duration_since(&last).unwrap().integer() as usize;

            let generic_duration = now.checked_duration_since(*last).unwrap();
            let microseconds = generic_duration.ticks();

            match recv.event(microseconds) {
                Ok(Some(cmd)) => {
                    defmt::info!("CMD: {:?}", defmt::Debug2Format(&cmd));
                }
                Ok(None) => {}
                Err(err) => defmt::error!("Recv error: {:?}", err),
            }
        }

        // Update Timestamp
        *prev = Some(now);

        // Toggle the led
        led.toggle();
    }
}
