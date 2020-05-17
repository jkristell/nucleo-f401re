#![no_std]

pub use stm32f4xx_hal as hal;
pub use hal::stm32 as pac;

mod led;
pub use led::Led;

mod button;
pub use button::Button;
