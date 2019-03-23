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


    let numbers: [u8; 10] = [0b1100_0000,
                            0b1111_1001,
                            0b1010_0100,
                            0b1011_0000,
                            0b1001_1001,
                            0b1001_0010,

                            0b1000_0010,
                            0b1111_1000,
                            0b1000_0000,
                            0b1001_1000,
    
    ];

    loop {

        let mut backbuf: [u8; 4] = [0; 4];

        backbuf[0] = numbers[1];
        backbuf[1] = numbers[2];
        backbuf[2] = numbers[3];
        backbuf[3] = numbers[4];

        let mut i: u8 = 0;
        let mut j: u8 = 0;
        loop {
            
            let buf = if(j == 0) {
                j = 1;
                [0,0]
            } else {
                j -= 1;
                i = i + 1 & 0b11;
                [backbuf[i as usize], 1 << i]
            };

            //let buf = if i & 1 == 1 {
            //    [0, 0]
            //} else {
            //    [backbuf[i as usize], 1 << i]
            //};



            write_buf(&mut spi, &mut latch, &mut delay, buf).unwrap();

            //delay.delay_ms(10_u16);

           // i = i + 1 & 0b11;

        }
    }


}


/*
    latch.set_low();
    block!(spi.send(0b1111_1001)).unwrap();
    let _ = block!(spi.read()).unwrap();
    block!(spi.send(0b0000_1111)).unwrap();
    let _ = block!(spi.read()).unwrap();
    delay.delay_ms(1u16);
    latch.set_high();
*/


    /*

    latch.set_low();
    block!(spi.send(0b0000_1101)).unwrap();
    block!(spi.send(0b0010_0001)).unwrap();
    latch.set_high();

    delay.delay_ms(10u16);
    //let buf: [u8; 2] = [0b0100_0000, 0b0000_1101];
    // let buf: [u8; 2] = [0xf0, 0x0A];
    // spi.write(&buf).unwrap();

    */
    /*
    block!(spi.send(0b0100_0000)).unwrap();
    let _ = block!(spi.read()).unwrap();
    block!(spi.send(0b0000_1101)).unwrap();
    let _ = block!(spi.read()).unwrap();
    */
    //spi.write(&buf).unwrap();
    //
    //

