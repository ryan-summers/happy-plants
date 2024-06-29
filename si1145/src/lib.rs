#![no_std]
use embedded_hal::i2c::I2c;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use core::convert::TryFrom;

#[derive(IntoPrimitive)]
#[repr(u8)]
enum Register {
    IntCfg = 0x03,
    IrqEn = 0x04,
    IrqMode1 = 0x05,
    IrqMode2 = 0x06,
    HwKey = 0x07,
    MeasRate0 = 0x08,
    MeasRate1 = 0x09,
    Ucoef0 = 0x13,
    Ucoef1 = 0x14,
    Ucoef2 = 0x15,
    Ucoef3 = 0x16,
    ParamWr = 0x17,
    Command = 0x18,
    Response = 0x20,
    IrqStat = 0x21,
    AlsVisData0 = 0x22,
    AlsIrData0 = 0x24,
    UvIndex0 = 0x2c,
}

#[derive(IntoPrimitive)]
#[repr(u8)]
enum Param {
    ChannelList = 0x01,
    AlsVisAdcMisc = 0x12,
    AlsIrAdcMisc = 0x1f,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum ResponseError {
    InvalidComand = 0x80,
    AdcOverflowPs1 = 0x88,
    AdcOverflowPs2 = 0x89,
    AdcOverflowPs3 = 0x8a,
    AdcOverflowAlsVis = 0x8c,
    AdcOverflowAlsIr = 0x8d,
    AdcOverflowAux = 0x8e,
}

struct ResponseRegister(pub u8);

impl ResponseRegister {
    pub fn as_result(self) -> Result<(), ResponseError> {
        let error_bits = self.0 & 0xF0;
        if let Ok(err) = ResponseError::try_from(error_bits) {
            Err(err)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error<T> {
    Interface(T),
    Response(ResponseError),
}

impl<T> From<T> for Error<T> {
    fn from(e: T) -> Self {
        Self::Interface(e)
    }
}

#[derive(Clone, Debug)]
pub struct Si1145<T> {
    addr: u8,
    device: T,
}

const DEFAULT_ADDR: u8 = 0x60;

impl<T> Si1145<T>
where
    T: I2c,
{
    pub fn new(
        device: T,
        delay: &mut impl embedded_hal::delay::DelayNs,
    ) -> Result<Self, Error<T::Error>> {
        let mut si114 = Self {
            addr: DEFAULT_ADDR,
            device,
        };

        si114.reset(delay)?;

        // TODO: Load factory calibration data for UV constants.
        Ok(si114)
    }

    pub fn measurement_ready(&mut self) -> Result<bool, Error<T::Error>> {
        let irq_status = self.read_reg(Register::IrqStat)?;
        self.write_reg(Register::IrqStat, irq_status)?;
        Ok(irq_status != 0)
    }

    pub fn reset(
        &mut self,
        delay: &mut impl embedded_hal::delay::DelayNs,
    ) -> Result<(), Error<T::Error>> {
        defmt::info!("SI1145 reset start");
        // Send a reset command to the chip.
        self.write_reg(Register::Command, 0x1)?;

        // The datasheet says to not perform any operations for at least 1 millisecond after reset.
        delay.delay_ms(2);

        self.write_reg(Register::MeasRate0, 0)?;
        self.write_reg(Register::MeasRate1, 0)?;
        self.write_reg(Register::IrqEn, 0b1)?;
        self.write_reg(Register::IrqMode1, 0)?;
        self.write_reg(Register::IrqMode2, 0)?;
        self.write_reg(Register::IntCfg, 0)?;
        self.write_reg(Register::IrqStat, 0xFF)?;

        // Initialize the HW_KEY to allow for normal chip operation.
        self.write_reg(Register::HwKey, 0x17)?;

        // Specify the UV measurement coeffients provided by Silicon labs in the datasheet.
        self.write_reg(Register::Ucoef0, 0x7B)?;
        self.write_reg(Register::Ucoef1, 0x6B)?;
        self.write_reg(Register::Ucoef2, 0x01)?;
        self.write_reg(Register::Ucoef3, 0x00)?;

        // Configure the measurement channel list to read UV, IR, and Visible light channels.
        self.write_parameter(Param::ChannelList, 0b1011_0000)?;

        // Configure the VIS and IR channels to use high range mode, which divides the ADC gain by
        // 14.5. All other parameters are left at default values for visible light measurement.
        // TODO: Does this impact lux conversion?
        self.write_parameter(Param::AlsVisAdcMisc, 0b0010_0000)?;
        self.write_parameter(Param::AlsIrAdcMisc, 0b0010_0000)?;

        // Configure the chip for auto-run mode with measurements at 8ms intervals.
        self.write_reg(Register::MeasRate0, 0xFF)?;

        self.write_reg(Register::Command, 0x0F)?;
        self.check_response_register()?;

        Ok(())
    }

    fn check_response_register(&mut self) -> Result<(), Error<T::Error>> {
        let response = ResponseRegister(self.read_reg(Register::Response)?).as_result();

        // Clear the error register by writing a NOP command.
        if response.is_err() {
            self.write_reg(Register::Command, 0)?;
        }

        response.map_err(Error::Response)?;
        Ok(())
    }

    fn write_parameter(&mut self, parameter: Param, value: u8) -> Result<(), Error<T::Error>> {
        self.write_reg(Register::ParamWr, value)?;
        self.write_reg(
            Register::Command,
            Into::<u8>::into(parameter) | 0b1010_0000_u8,
        )?;

        self.check_response_register()?;
        Ok(())
    }

    fn write_reg(&mut self, reg: Register, value: u8) -> Result<(), Error<T::Error>> {
        self.device.write(self.addr, &[reg.into(), value])?;
        Ok(())
    }

    fn read_reg_u16(&mut self, register: Register) -> Result<u16, Error<T::Error>> {
        let mut bytes = [0u8; 2];
        self.device
            .write_read(self.addr, &[register.into()], &mut bytes[..])?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_reg(&mut self, register: Register) -> Result<u8, Error<T::Error>> {
        let mut result = [0u8; 1];
        self.device
            .write_read(self.addr, &[register.into()], &mut result[..])?;
        Ok(result[0])
    }

    pub fn read_uv_index(&mut self) -> Result<f32, Error<T::Error>> {
        let uv_register = self.read_reg_u16(Register::UvIndex0)?;
        Ok(uv_register as f32 / 100.0)
    }

    pub fn read_visible(&mut self) -> Result<u16, Error<T::Error>> {
        self.read_reg_u16(Register::AlsVisData0)
    }

    pub fn read_infrared(&mut self) -> Result<u16, Error<T::Error>> {
        self.read_reg_u16(Register::AlsIrData0)
    }

    pub fn read_lux(&mut self) -> Result<f32, Error<T::Error>> {
        // ADC codes represent values of 256 and lower as "dark" or "negative" light. Thus, the
        // zero point is at 256 ADC counts.
        let als_vis = self
            .read_reg_u16(Register::AlsVisData0)?
            .saturating_sub(256);
        let als_ir = self.read_reg_u16(Register::AlsIrData0)?.saturating_sub(256);

        // The equation here is taken from AN523 section 6. The coefficinents in use assume there
        // is no coverglass over the sensor.
        Ok((5.41 * als_vis as f32 * 14.5) + (-0.08 * als_ir as f32))
    }
}
