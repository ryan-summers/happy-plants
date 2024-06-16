#![no_std]

//! I²C driver for the Silicon Labs [Si7021] hygrometer and thermometer.
//!
//! [Si7021]: https://www.silabs.com/documents/public/data-sheets/Si7021-A20.pdf

/// Standard I²C address of the Si7021: `0x40`
pub const SI7021_I2C_ADDRESS: u8 = 0x40;

// Some of the supported commands
// currently missing: accuracy control, heater, reset, async interface
const MEASURE_RELATIVE_HUMIDITY: u8 = 0xE5;
const MEASURE_TEMPERATURE: u8 = 0xE3;
const READ_TEMPERATURE: u8 = 0xE0;

/// Read temperature and relative humidity from a Si7021
#[derive(Clone, Debug)]
pub struct Si7021<T> {
    device: T,
}

impl<T> Si7021<T>
where
    T: embedded_hal::i2c::I2c,
{
    /// Create a new instance wrapping the given `I2CDevice`.
    pub fn new(device: T) -> Si7021<T> {
        Si7021 { device }
    }

    /// Every humidity measurement measures the temperature first. Use this
    /// function to read the most recently measured temperature.
    pub fn last_temperature(&mut self) -> Result<f32, T::Error> {
        let raw_temperature = self.read_word(READ_TEMPERATURE)?;

        Ok(calculate_temperature(raw_temperature))
    }

    fn read_word(&mut self, command: u8) -> Result<u16, T::Error> {
        let mut buf = [0u8; 2];
        self.device
            .write_read(SI7021_I2C_ADDRESS, &[command], &mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    pub fn relative_humidity(&mut self) -> Result<f32, T::Error> {
        let raw_humidity = self.read_word(MEASURE_RELATIVE_HUMIDITY)?;

        Ok(calculate_relative_humidity(raw_humidity))
    }

    pub fn temperature_celsius(&mut self) -> Result<f32, T::Error> {
        let raw_temperature = self.read_word(MEASURE_TEMPERATURE)?;

        Ok(calculate_temperature(raw_temperature))
    }
}

fn calculate_relative_humidity(raw_humidity: u16) -> f32 {
    let relative_humidity = 125.0 * raw_humidity as f32 / 65536.0 - 6.0;
    relative_humidity.max(0.0).min(100.0) // clamp as per datasheet
}

fn calculate_temperature(raw_temperature: u16) -> f32 {
    175.72 * raw_temperature as f32 / 65536.0 - 46.85
}
