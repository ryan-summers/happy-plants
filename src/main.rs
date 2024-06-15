#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{twim::{self, Twim}, bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

use si7021::Si7021;

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let config = twim::Config::default();
    let mut twi = Twim::new(p.TWISPI0, Irqs, p.P0_13, p.P0_15, config);

    defmt::info!("Initializing SI7021");
    let mut si7021 = Si7021::new(twi);
    defmt::info!("SI7021 initialized");

    loop {
        let temperature = si7021.temperature_celsius().unwrap();
        let humidity = si7021.relative_humidity().unwrap();
        defmt::info!("Temp: {=f32} Humidity: {=f32}", temperature, humidity);
    }
}
