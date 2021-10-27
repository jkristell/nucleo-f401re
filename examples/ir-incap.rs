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
    //let clocks = rcc.cfgr.freeze();

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
    let monitor = Timer::new(p.TIM1, &clocks).input_capture(1343, 62499, pwm_reader_ch1);

    defmt::debug!("timer freq = {:?}, psc = {:?} arr = {:?}",
        monitor.clk.0, monitor.psc,
        monitor.tim.arr.read().bits(),
    );

    let c = monitor.cc1();
    defmt::debug!("c = {:?}", c);



    cortex_m::interrupt::free(|cs| {
        //LED.borrow(cs).replace(Some(board_led));
        //IR_RX.borrow(cs).replace(Some(receiver));
        //TIMER.borrow(cs).replace(Some(monotonic));
        INCAP.borrow(cs).replace(Some(monitor));
    });

    // Enable interrupt on input pin
    unsafe { cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM1_CC) }

    defmt::info!("Setup done");

    loop {}
}

#[interrupt]
fn TIM1_CC() {
    static mut rising: u16 = 0;
    static mut falling: u16 = 0;

    cortex_m::interrupt::free(|cs| {

        let mut timer = INCAP.borrow(cs).borrow_mut();
        let timer = timer.as_mut().unwrap();

        let caps = timer.captures();
        //defmt::debug!("caps = {:?}", caps);

        if let Some(r) = caps.0 {
            defmt::debug!("diff = {:?}",
                r.wrapping_sub(*rising),
            );
                *rising = r;
        }

        //if let Some(f) = caps.1 {
        //    defmt::debug!("diff = {:?}",
        //        f.wrapping_sub(*rising),
        //    );
        //    *falling = f;
        //}

        //let c = caps.0.unwrap_or_default();
        //let diff = c.wrapping_sub(*last);
        //*last = c;
        //defmt::debug!("c = {:?}, diff = {:?}", c, diff);
    });
}