#![no_main]
#![no_std]

use core::cell::{RefCell};

use cortex_m::{interrupt::Mutex, peripheral::Peripherals};
use cortex_m_rt::entry;

use defmt_rtt as _;
use panic_probe as _;

use nucleo_f401re::{
    hal::{
        interrupt,
        prelude::*,
    },
    pac, Led,
};

use stm32f4xx_hal::timer::Timer;
use nucleo_f401re::hal::pwm_input::InputCapture;
use nucleo_f401re::hal::pac::TIM1;
use stm32f4xx_hal::gpio::Alternate;
use nucleo_f401re::hal::gpio::Pin;

//type IrProto = NecApple;
//type IrRemote = Apple2009;
//type IrReceivePin = PA10<Input<Floating>>;
//type IrReceiver = infrared::Receiver<IrProto, Event, PinInput<IrReceivePin>, Button<IrRemote>>;
type InCap = InputCapture<TIM1, Pin<Alternate<1>, 'A', 8>>;

static INCAP: Mutex<RefCell<Option<InCap>>> = Mutex::new(RefCell::new(None));
//static IR_RX: Mutex<RefCell<Option<IrReceiver>>> = Mutex::new(RefCell::new(None));
//static TIMER: Mutex<RefCell<Option<MonoTimer>>> = Mutex::new(RefCell::new(None));
//static LED: Mutex<RefCell<Option<Led>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let _cp = Peripherals::take().unwrap();

    let _syscfg = p.SYSCFG.constrain();
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpioa = p.GPIOA.split();
    let _gpiob = p.GPIOB.split();

    let _board_led = Led::new(gpioa.pa5);

    //let channels = (gpioa.pa8.into_alternate(), gpioa.pa9.into_alternate());
    // configure tim1 as a PWM output of known frequency.
    //let pwm = Timer::new(p.TIM1, &clocks).pwm(channels, 501u32.hz());
    //let (mut ch1, _ch2) = pwm;
    //let max_duty = ch1.get_max_duty();
    //ch1.set_duty(max_duty / 2);
    //ch1.enable();

    // Configure a pin into TIM8_CH1 mode, which will be used to observe an input PWM signal.
    let pwm_reader_ch1 = gpioa.pa8.into_alternate();

    // configure tim8 as a PWM input, using the best-guess frequency of the input signal.
    let monitor = Timer::new(p.TIM1, &clocks).input_capture( pwm_reader_ch1);

    // NOTE: this value may only be accurately observed at the CC2 interrupt.
    let _duty = monitor.cc1();



    cortex_m::interrupt::free(|cs| {
        //LED.borrow(cs).replace(Some(board_led));
        //IR_RX.borrow(cs).replace(Some(receiver));
        //TIMER.borrow(cs).replace(Some(monotonic));
        INCAP.borrow(cs).replace(Some(monitor));
    });

    // Enable interrupt on input pin
    unsafe { cortex_m::peripheral::NVIC::unmask(pac::Interrupt::EXTI15_10) }

    defmt::info!("Setup done");

    loop {}
}

#[interrupt]
fn TIM1_CC() {


    cortex_m::interrupt::free(|cs| {

        let mut timer = INCAP.borrow(cs).borrow_mut();
        let timer = timer.as_mut().unwrap();

        let c = timer.cc1();

        defmt::debug!("c = {:?}", c);

        /*
        let mut timer = TIMER.borrow(cs).borrow_mut();
        let mono = timer.as_mut().unwrap();

        let mut receiver = IR_RX.borrow(cs).borrow_mut();
        let receiver = receiver.as_mut().unwrap();

        if let Some(dt) = LAST.map(|i| i.elapsed()) {
            defmt::trace!("dt: {}", dt);

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
    */
    });
}