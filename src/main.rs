#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts, peripherals,
    twim::{self, Twim},
};
use {defmt_rtt as _, panic_probe as _};

use core::cell::RefCell;
use embedded_hal_bus::i2c::RefCellDevice;
use si1145::Si1145;
use si7021::Si7021;

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let config = twim::Config::default();
    let twi = Twim::new(p.TWISPI0, Irqs, p.P0_13, p.P0_15, config);
    let twi_ref_cell = RefCell::new(twi);

    defmt::info!("Initializing sensors");
    let mut si7021 = Si7021::new(RefCellDevice::new(&twi_ref_cell));
    defmt::info!("SI7021 initialized");

    let mut si1145 = Si1145::new(RefCellDevice::new(&twi_ref_cell)).unwrap();
    defmt::info!("SI1145 initialized");

    loop {
        let temperature = si7021.temperature_celsius().unwrap();
        let humidity = si7021.relative_humidity().unwrap();
        //let visible_lux = 0f32; //si1145.read_visible().unwrap();
        //let ir_lux = 0f32; //si1145.read_visible().unwrap();
        let visible_counts = si1145.read_visible().unwrap();
        let ir_counts = si1145.read_visible().unwrap();
        let illumination_lux = si1145.read_lux().unwrap();
        let uv_index = si1145.read_uv_index().unwrap();
        defmt::info!("Temp: {=f32} *C, Humidity: {=f32}%, Illum: {=f32} lx, UV index: {=f32}, Vis: {=u16:X}, IR: {=u16:X}", temperature, humidity, illumination_lux, uv_index, visible_counts, ir_counts);
    }
}
