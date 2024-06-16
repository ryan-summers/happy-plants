#![no_std]
use embedded_hal::i2c::I2c;
use num_enum::{IntoPrimitive, TryFromPrimitive};

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

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum ResponseError {
    InvalidComand = 0x80,
    AdcOverflowPs1 = 0x88,
    AdcOverflowPs2 = 0x89,
    AdcOverflowPs3 = 0x8a,
    AdcOverflowAlsVis = 0x8c,
    AdcOverflowAlsIr = 0x8d,
    AdcOverflowAux = 0x8e,
}

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
    pub fn new(device: T) -> Result<Self, Error<T::Error>> {
        let mut si114 = Self {
            addr: DEFAULT_ADDR,
            device,
        };

        si114.reset()?;

        // TODO: Load factory calibration data for UV constants.
        Ok(si114)
    }

    pub fn reset(&mut self) -> Result<(), Error<T::Error>> {
        self.write_reg(Register::MeasRate0, 0)?;
        self.write_reg(Register::MeasRate1, 0)?;
        self.write_reg(Register::IrqEn, 0)?;
        self.write_reg(Register::IrqMode1, 0)?;
        self.write_reg(Register::IrqMode2, 0)?;
        self.write_reg(Register::IntCfg, 0)?;
        self.write_reg(Register::IrqStat, 0xFF)?;

        // Send a reset command to the chip.
        self.write_reg(Register::Command, 0x1)?;
        // TODO: Verify the response is valid.
        let _response = self.read_reg(Register::Response)?;

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
        // TODO: Verify the response is valid.
        let _response = self.read_reg(Register::Response)?;

        Ok(())
    }

    fn write_parameter(&mut self, parameter: Param, value: u8) -> Result<(), Error<T::Error>> {
        self.write_reg(Register::ParamWr, value)?;
        self.write_reg(Register::Command, parameter.into())?;

        // TODO: Verify response.
        let _response = self.read_reg(Register::Response)?;
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

    pub fn read_visible(&mut self) -> Result<f32, Error<T::Error>> {
        let visible_register = self.read_reg_u16(Register::AlsVisData0)?;

        // The datasheet specifies 0.282 ADC counts / lux for sunlight response with ADC_GAIN = 0
        // and VIS_RANGE = 0. We use high range mode, so further compensate for the lower ADC gain.
        Ok(visible_register as f32 / 0.282 * 14.5)
    }

    pub fn read_infrared(&mut self) -> Result<f32, Error<T::Error>> {
        let ir_register = self.read_reg_u16(Register::AlsVisData0)?;
        // The datasheet specifies 2.44 ADC counts / lux. We use high range mode, so compensate for
        // the 14.5 ADC gain reduction.
        Ok(ir_register as f32 / 2.44 * 14.5)
    }
}
