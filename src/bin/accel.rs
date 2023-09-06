#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::can::{
    bxcan::{filter::Mask32, Fifo, Frame, StandardId},
    Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler,
};
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::CAN1;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};

use embassy_time::{Duration, Timer};
use embedded_hal::spi;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
    I2C1_EV => i2c::InterruptHandler<peripherals::I2C1>;
});

// spi words
const BIT_MASK_SPI_CMD_READ: u8 = 0x80;
const ACC_REG_DATA_START: u8 = 0x02;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut led = Output::new(p.PC5, Level::High, Speed::Low);
    let mut can = Can::new(p.CAN1, p.PB8, p.PB9, Irqs);

    // setup can
    can.as_mut()
        .modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.as_mut().modify_config().leave_disabled();

    can.set_bitrate(100_000);

    can.enable().await;

    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(100_000);
    spi_config.mode = spi::MODE_0;

    let mut spi = Spi::new(p.SPI1, p.PA5, p.PA7, p.PA6, NoDma, NoDma, spi_config);
    // set digital out pins
    let mut accel_active = Output::new(p.PA4, Level::High, Speed::Low);
    // let mut gyro_active = Output::new(p.PC13, Level::High, Speed::Low);
    // let mut mag_active = Output::new(p.PA1, Level::High, Speed::Low);

    let mut data_buffer: [u8; 6] = [0; 6];
    let read_request = [ACC_REG_DATA_START | BIT_MASK_SPI_CMD_READ];
    loop {
        led.toggle();

        // read accel data over spi
        accel_active.set_low();
        spi.blocking_write(&read_request).unwrap();
        spi.blocking_read(&mut data_buffer).unwrap();
        accel_active.set_high();

        // transmit accel data over can
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(0x123)), data_buffer);
        can.write(&tx_frame).await;
        Timer::after(Duration::from_millis(1000)).await;
    }
}
