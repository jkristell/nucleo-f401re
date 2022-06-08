use stm32f4xx_hal::{
    gpio::{gpioc::PC13, Edge, ExtiPin, Input, PinMode},
    pac::EXTI,
    syscfg::SysCfg,
};

pub struct Button {
    pin: PC13<Input>,
}

impl Button {
    pub fn new(pc13: PC13<impl PinMode>) -> Self {
        let pin = pc13.into_input().internal_pull_down(true);
        Self { pin }
    }

    pub fn enable_interrupt(&mut self, edge: Edge, syscfg: &mut SysCfg, exti: &mut EXTI) {
        // Enable external interrupt on PC13
        self.pin.make_interrupt_source(syscfg);
        self.pin.enable_interrupt(exti);
        self.pin.trigger_on_edge(exti, edge);
    }

    pub fn clear_interrupt_pending_bit(&mut self) {
        self.pin.clear_interrupt_pending_bit();
    }
}
