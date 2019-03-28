#![no_main]
#![no_std]

extern crate nucleo_f401re as board;
extern crate panic_semihosting;

use board::hal::gpio::{
    gpioa::PA5,
    gpiob::{PB3, PB4, PB5},
    gpioc::PC13,
    Alternate, Edge, ExtiPin, Input, Output, PullDown, PushPull, AF5,
};
use board::hal::stm32::{self, SPI1};
use board::prelude::*;
use board::spi::{self, Spi};

use cortex_m_semihosting::hprintln;
use rtfm::{app, Instant};

use segment_display::SegmentDisplay;

const CPU_FREQ: u32 = 84_000_000;

pub enum State {
    Paus,
    Play,
    CountUp,
    CountDown,
}

impl State {
    pub fn next(&mut self) {
        use State::*;
        *self = match self {
            Paus => Play,
            Play => CountUp,
            CountUp => CountDown,
            CountDown => CountUp,
        };
    }
}

#[app(device = board::hal::stm32)]
const APP: () = {
    // Late resources
    static mut EXTI: stm32::EXTI = ();
    static mut BUTTON: PC13<Input<PullDown>> = ();
    static mut LED: PA5<Output<PushPull>> = ();
    static mut DISPLAY: SegmentDisplay<Spi<SPI1, (PB3<Alternate<AF5>>, spi::NoMiso, PB5<Alternate<AF5>>)>, PB4<Output<PushPull>>> = ();
    static mut COUNTER: usize = 0;
    static mut STATE: State = State::Paus;

    #[init(schedule = [refresh, write_display])]
    fn init() {
        // Cortex-M peripherals
        let _core: rtfm::Peripherals = core;

        // Device specific peripherals
        let mut device: stm32::Peripherals = device;

        // Configure PC13 (User Button) as an input
        let gpioc = device.GPIOC.split();
        let mut button = gpioc.pc13.into_pull_down_input();

        // Configure the led pin as an output
        let gpioa = device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Enable the clock for the SYSCFG
        device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

        // Setup the system clock
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(CPU_FREQ.hz()).freeze();

        // Enable interrupt on PC13
        button.make_interrupt_source(&mut device.SYSCFG);
        button.enable_interrupt(&mut device.EXTI);
        button.trigger_on_edge(&mut device.EXTI, Edge::FALLING);

        // Configure the SPI
        let gpiob = device.GPIOB.split();
        let sck = gpiob.pb3.into_alternate_af5();
        //let miso = gpiob.pb4.into_alternate_af5();
        let miso = spi::NoMiso;
        let mosi = gpiob.pb5.into_alternate_af5();

        // rclk
        let latch = gpiob.pb4.into_push_pull_output();

        let mode = spi::Mode {
            polarity: spi::Polarity::IdleHigh,
            phase: spi::Phase::CaptureOnFirstTransition,
        };

        let spi = Spi::spi1(device.SPI1, (sck, miso, mosi), mode, 4_000_000.hz(), clocks);

        let segment_display = SegmentDisplay::new(spi, latch);

        let now = Instant::now();
        // The segment refresher
        schedule.refresh(now + (CPU_FREQ / 8000).cycles()).unwrap();

        schedule.write_display(now + (CPU_FREQ / 10).cycles()).unwrap();

        hprintln!("init done").unwrap();

        EXTI = device.EXTI;
        LED = led;
        BUTTON = button;
        DISPLAY = segment_display;
    }

    #[idle(resources = [DISPLAY])]
    fn idle() -> ! {
        hprintln!("idle").unwrap();

        // The idle loop
        loop {}
    }

    #[task(schedule = [write_display], resources = [DISPLAY, COUNTER, STATE])]
    fn write_display() {
        match *resources.STATE {
            State::Paus => resources.DISPLAY.write_str("PAUS"),
            State::Play => resources.DISPLAY.write_str("PLAY"),
            State::CountUp => {
                resources.DISPLAY.write_number(*resources.COUNTER);
                if *resources.COUNTER == 9999 {
                    *resources.COUNTER = 0;
                } else {
                    *resources.COUNTER += 1;
                }
            }
            State::CountDown => {
                resources.DISPLAY.write_number(*resources.COUNTER);
                if *resources.COUNTER == 0 {
                    *resources.COUNTER = 9999;
                } else {
                    *resources.COUNTER -= 1;
                }
            }
        };

        schedule
            .write_display(scheduled + (CPU_FREQ / 10).cycles())
            .unwrap();
    }

    #[task(schedule = [refresh], resources = [DISPLAY])]
    fn refresh() {
        resources.DISPLAY.refresh().unwrap();
        schedule
            .refresh(scheduled + (CPU_FREQ / 1000).cycles())
            .unwrap();
    }

    #[interrupt(binds = EXTI15_10, resources = [EXTI, LED, BUTTON, DISPLAY, COUNTER, STATE])]
    fn on_button_press() {
        // Clear the interrupt
        resources.BUTTON.clear_interrupt_pending_bit(resources.EXTI);

        resources.STATE.next();

        // Toggle the led
        resources.LED.toggle();
    }

    extern "C" {
        fn EXTI0();
    }
};
