#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_semihosting;
extern crate nucleo_f401re as board;

use cortex_m_rt::entry;

use board::hal::prelude::*;
use board::hal::stm32;
use board::spi::{self, Spi};
use board::Interrupt;
use board::hal::{delay::Delay};
use cortex_m::peripheral::Peripherals;

use embedded_hal::blocking::spi as hal_spi;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::OutputPin;

//use board::hal::spi;

use nb::block;

struct SegmentDisp<'a, SPI, LATCH> {
    back_buffer : [u8;4],
    spi : &'a mut SPI,
    latch : &'a mut LATCH,
    current_segment : u8,
}

impl<'a, SPI, LATCH> SegmentDisp<'a, SPI, LATCH>
    where SPI : hal_spi::Write<u8>,
          LATCH: OutputPin,
{

    pub fn new(spi : &'a mut SPI, latch : &'a mut LATCH) -> Self {
        Self {back_buffer : [0;4], spi, latch, current_segment : 0}
    }

    pub fn refresh<DELAY>(&mut self, delay :  &mut DELAY) -> Result<(), SPI::Error>
        where DELAY : DelayUs<u16>,
    {
        let curr_buff:[u8;2] = [self.back_buffer[self.current_segment as usize], 1 << self.current_segment];

        self.current_segment = if (self.current_segment == 3) {
            0
        } else {
            self.current_segment + 1
        };


        self.latch.set_low();

        let r = self.spi.write(&curr_buff);
        delay.delay_us(500);

        self.latch.set_high();

        r
    }

    pub fn set_buf(&mut self, buf:[char; 4])
    {
        for (i, c) in buf.iter().enumerate() {
            self.back_buffer[i] = Self::char_to_segment_code(*c);
        }
    }

    fn char_to_segment_code(c : char) -> u8 {
        match c {
            '0' => 0b1100_0000,
            '1' => 0b1111_1001,
            '2' => 0b1010_0100,
            '3' => 0b1011_0000,
            '4' => 0b1001_1001,
            '5' => 0b1001_0010,
            '6' => 0b1000_0010,
            '7' => 0b1111_1000,
            '8' => 0b1000_0000,
            '9' => 0b1001_1000,
            '.' => 0b0111_1111,
            _ => 0x00
        }
    } 
}


fn write_buf<SPI, LATCH, DELAY>(spi: &mut SPI, 
                                latch: &mut LATCH,
                                delay: &mut DELAY, 
                                buf: [u8; 2]) -> Result<(), SPI::Error>
where SPI: hal_spi::Write<u8>,
      LATCH: OutputPin,
      DELAY: DelayUs<u16>,
{
    latch.set_low();

    let r = spi.write(&buf);
    delay.delay_us(500);

    latch.set_high();

    r
}

#[entry]
fn main() -> ! {
    // The Stm32 peripherals
    let mut device = stm32::Peripherals::take().unwrap();
    // The Cortex-m peripherals
    let mut core = Peripherals::take().unwrap();

    let _gpioa = device.GPIOA.split();
    let gpiob = device.GPIOB.split();

    // Enable the clock for the SYSCFG
    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    // Constrain clock registers
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

     let mut delay = Delay::new(core.SYST, clocks);

    let sck = gpiob.pb3.into_alternate_af5();
    //let miso = gpiob.pb4.into_alternate_af5();
    let miso = spi::NoMiso;
    let mosi = gpiob.pb5.into_alternate_af5();

    // rclk
    let mut latch = gpiob.pb4.into_push_pull_output();


    let mode = spi::Mode {
        polarity: spi::Polarity::IdleHigh,
        phase: spi::Phase::CaptureOnFirstTransition,
    };

    let mut spi = Spi::spi1(device.SPI1, (sck, miso, mosi), mode, 100_000.hz(), clocks);

    let mut segment_display = SegmentDisp::new(&mut spi, &mut latch);
    segment_display.set_buf(['4','2','4','2']);

    loop {
        segment_display.refresh(&mut delay);
    }


}


