#![no_main]
#![no_std]

use core::cell::RefCell;

use cortex_m::{interrupt::Mutex, peripheral::Peripherals};
use cortex_m_rt::entry;

use defmt_rtt as _;
use panic_probe as _;

use nucleo_f401re::{
    hal::{
        gpio::{gpioa::PA10, Edge, Floating, Input},
        interrupt,
        prelude::*,
    },
    pac, Led,
};

use infrared::protocol::NecApple;
use infrared::remotecontrol::nec::Apple2009;
use infrared::remotecontrol::Button;
use infrared::{
    receiver::{Event, PinInput},
    Receiver,
};
use stm32f4xx_hal::dwt::{Instant, MonoTimer};

type IrProto = NecApple;
type IrRemote = Apple2009;
type IrReceivePin = PA10<Input<Floating>>;
type IrReceiver = infrared::Receiver<IrProto, Event, PinInput<IrReceivePin>, Button<IrRemote>>;

static IR_RX: Mutex<RefCell<Option<IrReceiver>>> = Mutex::new(RefCell::new(None));
static TIMER: Mutex<RefCell<Option<MonoTimer>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Led>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = Peripherals::take().unwrap();

    let mut syscfg = p.SYSCFG.constrain();
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();

    let monotonic = MonoTimer::new(cp.DWT, cp.DCB, &clocks);
    let mono_freq = monotonic.frequency();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();

    let board_led = Led::new(gpioa.pa5);

    // Leds on the daughter board
    let mut led_yellow = gpioa.pa8.into_push_pull_output();
    led_yellow.set_high();

    let mut led_blue = gpiob.pb10.into_push_pull_output();
    led_blue.set_high();

    let mut ir_recv_pin = gpioa.pa10;
    ir_recv_pin.make_interrupt_source(&mut syscfg);
    ir_recv_pin.enable_interrupt(&mut p.EXTI);
    ir_recv_pin.trigger_on_edge(&mut p.EXTI, Edge::RisingFalling);

    let receiver = Receiver::builder()
        .protocol::<IrProto>()
        .remotecontrol(IrRemote::default())
        .resolution(mono_freq.to_Hz())
        .event_driven()
        .pin(ir_recv_pin)
        .build();

    cortex_m::interrupt::free(|cs| {
        LED.borrow(cs).replace(Some(board_led));
        IR_RX.borrow(cs).replace(Some(receiver));
        TIMER.borrow(cs).replace(Some(monotonic));
    });

    // Enable interrupt on input pin
    unsafe { cortex_m::peripheral::NVIC::unmask(pac::Interrupt::EXTI15_10) }

    defmt::info!("Setup done");

    loop {}
}

#[interrupt]
fn EXTI15_10() {
    static mut LAST: Option<Instant> = None;

    cortex_m::interrupt::free(|cs| {
        let mut timer = TIMER.borrow(cs).borrow_mut();
        let mono = timer.as_mut().unwrap();

        let mut receiver = IR_RX.borrow(cs).borrow_mut();
        let receiver = receiver.as_mut().unwrap();

        if let Some(dt) = LAST.map(|i| i.elapsed()) {
            match receiver.event(dt) {
                Ok(Some(cmd)) => {
                    defmt::info!("cmd: {:?}", cmd);
                }
                Ok(None) => {}
                Err(err) => defmt::error!("Receiver error: {:?}", err),
            }
        }

        LAST.replace(mono.now());

        receiver.pin().clear_interrupt_pending_bit();

        let mut led = LED.borrow(cs).borrow_mut();
        led.as_mut().unwrap().toggle();
    });
}
