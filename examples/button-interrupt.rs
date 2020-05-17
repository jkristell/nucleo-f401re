#![no_main]
#![no_std]

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target;

use nucleo_f401re::{
    Led, Button,
    gpio::{Edge},
    hal::{
        interrupt,
    },
    prelude::*,
    stm32, Interrupt,
};

// Used to signal to the main loop that it should toggle the led
static SIGNAL: AtomicBool = AtomicBool::new(false);

static BUTTON: Mutex<RefCell<Option<Button>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    // The Stm32 peripherals
    let mut device = stm32::Peripherals::take().unwrap();
    // The Cortex-m peripherals
    let _core = Peripherals::take().unwrap();

    // Enable the clock for the SYSCFG
    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());
    // Constrain clock registers
    let rcc = device.RCC.constrain();
    let _clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpioa = device.GPIOA.split();
    let gpioc = device.GPIOC.split();

    // Configure PA5 (LD2 - User led) as an output
    let mut led = Led::new(gpioa.pa5);

    // Configure PC5 (User B1) as an input and enable external interrupt
    let mut button = Button::new(gpioc.pc13);
    button.enable_interrupt(Edge::RISING, &mut device.SYSCFG, &mut device.EXTI);

    cortex_m::interrupt::free(|cs| {
        BUTTON.borrow(cs).replace(Some(button));
    });

    // Enable the external interrupt
    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI15_10);
    }

    loop {
        let state_change = SIGNAL.load(Ordering::SeqCst);
        if state_change {
            led.toggle();
            SIGNAL.store(false, Ordering::SeqCst);
        }
    }
}

#[interrupt]
fn EXTI15_10() {
    // Clear the interrupt
    cortex_m::interrupt::free(|cs| {
        let mut button = BUTTON.borrow(cs).borrow_mut();
        button.as_mut().unwrap().clear_interrupt_pending_bit();
    });

    SIGNAL.store(true, Ordering::SeqCst);
}
