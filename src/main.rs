#![no_std]
#![no_main]

// pick a panicking behavior
// extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
// use cortex_m_semihosting::{hprintln};

extern crate panic_halt;

use core::fmt::Write;
use cortex_m_rt::entry;
use si7021::Si7021;

use nrf52840_hal::{
    uarte,
    twim,
    uarte::{Parity, Baudrate},
    gpio::{GpioExt, Level},
    nrf52840_pac::Peripherals
};


#[entry]
fn main() -> ! {

    let peripherals = Peripherals::take().unwrap();

    //{
    //    peripherals.CLOCK.tasks_hfclkstart.write(|w| unsafe {w.bits(1)});
    //    while peripherals.CLOCK.events_hfclkstarted.read().bits() == 0 {}
    //}

    let p0 = peripherals.P0.split();

    let mut uart = {

        let rxd = p0.p0_08.into_floating_input().degrade();
        let txd = p0.p0_06.into_push_pull_output(Level::Low).degrade();

        let pins = uarte::Pins{
            rxd: rxd,
            txd: txd,
            cts: None,
            rts: None
        };

        uarte::Uarte::new(
            peripherals.UARTE0,
            pins,
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        )
    };

    let i2c = {
        let sda = p0.p0_13.into_floating_input().degrade();
        let scl = p0.p0_15.into_floating_input().degrade();

        let pins = twim::Pins{
            scl: scl,
            sda: sda
        };

        twim::Twim::new(
            peripherals.TWIM0,
            pins,
            twim::Frequency::K100
        )
    };

    let mut si7021 = Si7021::new(i2c);

    writeln!(uart, "UART initialized\n").unwrap();

    loop {
        let temperature = si7021.temperature_celsius().unwrap();
        let humidity = si7021.relative_humidity().unwrap();
        writeln!(uart, "Temp: {:.2} Humidity: {:.2}", temperature, humidity).unwrap();
    }
}
