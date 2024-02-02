// this started as a copy of the example from embassy-stm32/examples/can.rs see acknowledgements file for more info
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{
        bxcan::{filter::Mask32, Fifo, Frame, StandardId},
        Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler,
    },
    gpio::{Level, Output, Speed},
    peripherals::CAN1,
};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PC5, Level::High, Speed::Low);
    let mut can = Can::new(p.CAN1, p.PB8, p.PB9, Irqs);

    can.as_mut()
        .modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.as_mut().modify_config().leave_disabled();

    can.set_bitrate(100_000);

    can.enable().await;

    let mut count: u8 = 10;
    loop {
        Timer::after(Duration::from_millis(1000)).await;
        for _ in 0..count {
            // blink the led "count" times
            led.set_high();
            Timer::after(Duration::from_millis(200)).await;
            led.set_low();
            Timer::after(Duration::from_millis(200)).await;
        }

        // read the count from the bus
        let envelope = can.read().await.unwrap();
        count = unwrap!(envelope.frame.data())[0];

        // increment the count
        count += 1;

        // reset the count if it gets too big
        count = if count > 5 { 0 } else { count };

        // send the count back
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(count as _)), [count]);
        can.write(&tx_frame).await;
    }
}
