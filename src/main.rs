#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts, peripherals,
    twim::{self, Twim},
};
use embedded_hal::delay::DelayNs;
use {defmt_rtt as _, panic_probe as _};

use core::cell::RefCell;
use embedded_hal_bus::i2c::RefCellDevice;
use sgp30::Sgp30;
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
    let mut delay = embassy_time::Delay;
    let twi_ref_cell = RefCell::new(twi);

    defmt::info!("Initializing sensors");
    let mut si7021 = Si7021::new(RefCellDevice::new(&twi_ref_cell));
    defmt::info!("SI7021 initialized");

    let mut si1145 = Si1145::new(RefCellDevice::new(&twi_ref_cell), &mut delay).unwrap();
    defmt::info!("SI1145 initialized");

    let mut sgp30 = Sgp30::new(RefCellDevice::new(&twi_ref_cell), 0x58, embassy_time::Delay);
    sgp30.init().unwrap();

    loop {
        if !si1145.measurement_ready().unwrap() {
            si1145.reset(&mut delay).unwrap();
        }

        let air_quality = sgp30.measure().unwrap();

        let temperature = si7021.temperature_celsius().unwrap();
        let humidity = si7021.relative_humidity().unwrap();

        // Conversion equation taken from https://carnotcycle.wordpress.com/2012/08/04/how-to-convert-relative-humidity-to-absolute-humidity/
        let abs_humidity = 6.122
            * libm::powf(
                core::f32::consts::E,
                17.67 * temperature / (temperature + 243.5),
            )
            * humidity
            * 2.1674
            / (273.15 + temperature);
        sgp30
            .set_humidity(Some(&sgp30::Humidity::from_f32(abs_humidity).unwrap()))
            .unwrap();

        let visible_counts = si1145.read_raw_visible().unwrap();
        let ir_counts = si1145.read_raw_infrared().unwrap();
        let illumination_lux = si1145.measure_lux().unwrap();
        let uv_index = si1145.measure_uv_index().unwrap();

        defmt::info!("environment,position=bedroom-nw-window temperature={=f32},humidity={=f32},abs-humidity={=f32},illumination={=f32},uv-index={=f32},co2-eq={=u16},tvoc={=u16},visible-adc={=u16},ir-adc={=u16}",
                     temperature,
                     humidity,
                     abs_humidity,
                     illumination_lux,
                     uv_index,
                     air_quality.co2eq_ppm,
                     air_quality.tvoc_ppb,
                     visible_counts,
                     ir_counts);
        delay.delay_ms(1000);
    }
}
