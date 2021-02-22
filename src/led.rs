use stm32f4xx_hal::gpio::{gpioa::PA5, Output, PushPull};

/// Onboard led
pub struct Led {
    pa5: PA5<Output<PushPull>>,
}

impl Led {
    pub fn new<M>(pin: PA5<M>) -> Self {
        let pa5 = pin.into_push_pull_output();
        Self { pa5 }
    }

    pub fn set(&mut self, enable: bool) {
        if enable {
            self.pa5.set_high();
        } else {
            self.pa5.set_low();
        }
    }

    pub fn toggle(&mut self) {
        self.pa5.toggle();
    }
}
