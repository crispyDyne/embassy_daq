#![no_std]
#![no_main]
// #![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PC5, Level::High, Speed::Low);
    let button = Input::new(p.PC11, Pull::Up);

    info!("Initialized");
    for _ in 0..10 {
        led.toggle();
        Timer::after(Duration::from_millis(100)).await;
    }

    let mut button_pressed = false;
    loop {
        if button.is_low() {
            if !button_pressed {
                info!("Button Pressed!");
                button_pressed = true;
            }
            led.set_high();
        } else {
            button_pressed = false;
            led.set_low();
        }
    }
}
