// http://download.mikroe.com/documents/datasheets/ZSC31014_datasheet.pdf
// https://www.renesas.com/us/en/document/dst/zsc31014-datasheet

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::CAN1;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_stm32::{
    can::{
        bxcan::{filter::Mask32, Fifo, Frame, StandardId},
        Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler,
    },
    i2c::I2c,
};

use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
    I2C1_EV => i2c::InterruptHandler<peripherals::I2C1>;
});

// i2c words
const SET_DEV_ADDR: u8 = 0x28;
const STATUS_BIT_MASK: u8 = 0x03;
const BRIDGE_RES: u16 = 0x3FFF;
const CMD_MODE_START: u8 = 0xA0;
const CMD_MODE_STOP: u8 = 0x80;

// eeprom i2c words
const EEPROM_BRIDGE_CONFIG: u8 = 0x0F;
const EEPROM_WRITE_CMD: u8 = 0x40;

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

    // configure i2c
    let mut i2c = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        NoDma,
        NoDma,
        Hertz(100_000),
        Default::default(),
    );

    // set digital out pins for mikroBus 2
    let loadcell_data_ready = Input::new(p.PB14, Pull::Down);
    let _loadcell_enable = Output::new(p.PB12, Level::High, Speed::Low); // must be high to enable load cell
    Timer::after(Duration::from_millis(3)).await;

    // initialize buffers
    let mut write_buffer: [u8; 3];
    let mut read_buffer: [u8; 3] = [0; 3];

    // put load cell into command mode
    write_buffer = [CMD_MODE_START, 0x00, 0x00];
    i2c.blocking_write(SET_DEV_ADDR, &write_buffer).unwrap();
    Timer::after(Duration::from_millis(10)).await;

    // read bridge config
    write_buffer = [EEPROM_BRIDGE_CONFIG, 0x00, 0x00];
    i2c.blocking_write(SET_DEV_ADDR, &write_buffer).unwrap();
    i2c.blocking_read(SET_DEV_ADDR, &mut read_buffer).unwrap();
    Timer::after(Duration::from_millis(1)).await;

    // transmit current bridge config over can
    let tx_frame = Frame::new_data(unwrap!(StandardId::new(0x123)), read_buffer);
    can.write(&tx_frame).await;

    // update bridge config
    let mut bridge_config =
        BridgeConfig::unpack_bridge_config(u8_array_to_u16([read_buffer[1], read_buffer[2]]));
    bridge_config.preamp_gain = 0b111; // 0b111 -> 192x gain

    let bridge_update = BridgeConfig::pack_bridge_config(&bridge_config);

    // write bridge config
    write_buffer = [
        EEPROM_BRIDGE_CONFIG | EEPROM_WRITE_CMD,
        (bridge_update >> 8) as u8,
        bridge_update as u8,
    ];
    i2c.blocking_write(SET_DEV_ADDR, &write_buffer).unwrap();
    Timer::after(Duration::from_millis(15)).await;

    // transmit updated bridge config over can
    let tx_frame = Frame::new_data(unwrap!(StandardId::new(0x123)), write_buffer);
    can.write(&tx_frame).await;

    // end command mode
    write_buffer = [CMD_MODE_STOP, 0x00, 0x00];
    i2c.blocking_write(SET_DEV_ADDR, &write_buffer).unwrap();
    Timer::after(Duration::from_millis(20)).await;

    loop {
        led.toggle();

        // await data ready
        while loadcell_data_ready.is_low() {
            Timer::after(Duration::from_millis(10)).await;
        }

        // read load data
        let (status_data, bridge_data, temperature_data) = read_data(&mut i2c);
        let bridge_u8a = u16_to_u8_array(bridge_data);
        let temperature_u8a = u16_to_u8_array(temperature_data as u16);

        // transmit accel data over can
        let tx_frame = Frame::new_data(
            unwrap!(StandardId::new(0x123)),
            [
                status_data,
                bridge_u8a[0],
                bridge_u8a[1],
                temperature_u8a[0],
                temperature_u8a[1],
            ],
        );
        can.write(&tx_frame).await;
        Timer::after(Duration::from_millis(100)).await;
    }
}

fn u16_to_u8_array(data: u16) -> [u8; 2] {
    let mut data_array: [u8; 2] = [0; 2];

    data_array[0] = (data >> 8) as u8;
    data_array[1] = data as u8;

    data_array
}

fn u8_array_to_u16(data: [u8; 2]) -> u16 {
    let mut data_u16: u16 = 0;

    data_u16 |= (data[0] as u16) << 8;
    data_u16 |= data[1] as u16;

    data_u16
}

fn read_raw<T: embassy_stm32::i2c::Instance, I>(i2c: &mut I2c<'_, T, I>) -> u32 {
    let mut rx_buf = [0u8; 4];

    match i2c.blocking_read(SET_DEV_ADDR, &mut rx_buf) {
        Ok(_res) => {}
        Err(_e) => {}
    }

    let tmp = ((rx_buf[0] as u32) << 24)
        | ((rx_buf[1] as u32) << 16)
        | ((rx_buf[2] as u32) << 8)
        | (rx_buf[3] as u32);

    tmp
}

fn read_data<T: embassy_stm32::i2c::Instance, I>(mut i2c: &mut I2c<'_, T, I>) -> (u8, u16, i16) {
    let raw_data = read_raw(&mut i2c);

    let status_data = ((raw_data >> 30) & (STATUS_BIT_MASK as u32)) as u8;
    let bridge_data = ((raw_data >> 16) & (BRIDGE_RES as u32)) as u16;
    let temperature_data = (raw_data >> 5) as i16;

    (status_data, bridge_data, temperature_data)
}

struct BridgeConfig {
    a2d_offset: u8,
    preamp_gain: u8,
    gain_polarity: bool,
    long_int: bool,
    bsink: bool,
    preamp_mux: u8,
    disable_nulling: bool,
    idt_reserved: u8,
}

impl BridgeConfig {
    fn unpack_bridge_config(bridge_config: u16) -> Self {
        let a2d_offset = (bridge_config & 0x000F) as u8;
        let preamp_gain = ((bridge_config >> 4) & 0x0007) as u8;
        let gain_polarity = ((bridge_config >> 7) & 0x0001) == 1;
        let long_int = ((bridge_config >> 8) & 0x0001) == 1;
        let bsink = ((bridge_config >> 9) & 0x0001) == 1;
        let preamp_mux = ((bridge_config >> 10) & 0x0003) as u8;
        let disable_nulling = ((bridge_config >> 12) & 0x0001) == 1;
        let idt_reserved = ((bridge_config >> 13) & 0x0007) as u8;

        BridgeConfig {
            a2d_offset,
            preamp_gain,
            gain_polarity,
            long_int,
            bsink,
            preamp_mux,
            disable_nulling,
            idt_reserved,
        }
    }

    fn pack_bridge_config(&self) -> u16 {
        let mut bridge_config: u16 = 0;

        bridge_config |= (self.a2d_offset as u16) & 0x000F;
        bridge_config |= ((self.preamp_gain as u16) << 4) & 0x0070;
        bridge_config |= ((self.gain_polarity as u16) << 7) & 0x0080;
        bridge_config |= ((self.long_int as u16) << 8) & 0x0100;
        bridge_config |= ((self.bsink as u16) << 9) & 0x0200;
        bridge_config |= ((self.preamp_mux as u16) << 10) & 0x0C00;
        bridge_config |= ((self.disable_nulling as u16) << 12) & 0x1000;
        bridge_config |= ((self.idt_reserved as u16) << 13) & 0xE000;

        bridge_config
    }
}
