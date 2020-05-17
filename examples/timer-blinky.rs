#![no_main]
#![no_std]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;
use panic_rtt_target as _;

use nucleo_f401re::{
    hal::{
        interrupt,
        prelude::*,
        timer::{Event, Timer},
    },
    pac, Led,
};

static TIMER: Mutex<RefCell<Option<Timer<pac::TIM2>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Led>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let p = pac::Peripherals::take().unwrap();
    let _core = Peripherals::take().unwrap();

    // Enable the clock for the SYSCFG
    p.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    let gpioa = p.GPIOA.split();

    let led = Led::new(gpioa.pa5);

    cortex_m::interrupt::free(|cs| {
        LED.borrow(cs).replace(Some(led));
    });

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    // Setup timer
    let mut timer = Timer::tim2(p.TIM2, 1.hz(), clocks);

    // Enable interrupt
    timer.listen(Event::TimeOut);

    cortex_m::interrupt::free(|cs| {
        TIMER.borrow(cs).replace(Some(timer));
    });

    // Enable TIM2 interrupt
    unsafe { cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2) }

    loop {}
}

#[interrupt]
fn TIM2() {
    // Ack the interrupt
    unsafe {
        (*pac::TIM2::ptr()).sr.modify(|_, w| w.uif().clear_bit());
    }

    // Toggle led
    cortex_m::interrupt::free(|cs| {
        let mut led = LED.borrow(cs).borrow_mut();
        led.as_mut().unwrap().toggle();
    });
}
