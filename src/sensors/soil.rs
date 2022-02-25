// use linux_embedded_hal::I2cdev;
use linux_embedded_hal::I2cdev;
use linux_embedded_hal::i2cdev::core::I2CDevice;
// use embedded_hal::blocking::i2c::Write;
use std::path::Path;

use crate::influx::Field;

pub struct SoilSensor {
    adc: ADS1115,
}


impl SoilSensor {
    pub fn new<P: AsRef<Path>>(path: P, address: u8) -> Self {
        let adc = ADS1115::new(path,address);
        Self { adc }
    }

    pub fn measure(&mut self) -> [Field<f32>;1] {
        let measurement = self.adc.measure();
        let m = (measurement as f32)/(u16::MAX as f32);
        [("MoistureContent", m).into()]
    }
}

/// Struct representing ADC between analog soil sensor
/// and raspberry pi
struct ADS1115 {
    i2c_device: I2cdev,
    address: u8,
    conf_reg_msb: u8,
    conf_reg_lsb: u8,
}

impl ADS1115 {
    pub fn new<P: AsRef<Path>>(path: P, address: u8) -> Self {
        let i2c_device = I2cdev::new(path).unwrap();
        let conf_reg_msb = 0b01000000;
        let conf_reg_lsb = 0b00000011;
        let mut s = Self { i2c_device, address, conf_reg_msb, conf_reg_lsb };
        s.init();
        s
    }

    fn init(&mut self) {
        self.write_to_config_reg(self.conf_reg_msb,self.conf_reg_lsb);
    }

    fn measure(&mut self) -> u16 {
        let mut msb = 0;
        let mut lsb = 0;
        self.read_conv_reg(&mut msb, &mut lsb);
        (msb as u16) << 8 | (lsb as u16)
    }
    fn set_channel(&mut self, channel: u8) {
        assert!(channel < 4);
        // clear bits
        self.conf_reg_msb ^= 0b01110000;
        self.conf_reg_msb |= 0b01000000 | (channel<<4) ;
        self.write_to_config_reg(self.conf_reg_msb,self.conf_reg_lsb);
    }

    fn write_to_config_reg(&mut self, data_msb: u8, data_lsb: u8) {
        // send address and aim pointer reg to conversion reg
        // then write data
        self.i2c_device.write(&[self.address,0b1,data_msb,data_lsb]).unwrap()
    }

    fn read_conv_reg(&mut self, data_msb: &mut u8, data_lsb: &mut u8) {
        // send address and aim pointer reg to conversion reg
        self.i2c_device.write(&[self.address, 0b0]).unwrap();
        // read 2 bytes
        let mut data = [0u8; 2];
        self.i2c_device.read(&mut data).unwrap();
    }

}

// struct ConversionRegister {

// }

// impl ConversionRegister {

// }

// /// Operational status/single-shot conversion start
// enum Os {
//     /// Begin conversion
//     BeginConv,
//     /// Do nothing
//     NoEffect,
// }

// /// Input multiplexer configuration
// enum Mux {
//   AinpAin0AndAinnAin1, // 000, default
//   AinpAin0AndAinnAin3, // 001
//   AinpAin1AndAinnAin3, // 010
//   AinpAin2AndAinnAin3, // 011
//   AinpAin0AndAinnGnd, // 100
//   AinpAin1AndAinnGnd, // 101
//   AinpAin2AndAinnGnd, // 110
//   AinpAin3AndAinnGnd, // 111
// }

// /// Programmable gain amplifier configuration
// enum Pga {
//     FsPm6_144v, // 000
//     FsPm4_096v, // 001
//     FsPm2_048v, // 010
//     FsPm1_024v, // 011
//     FsPm0_512v, // 100
//     FsPm0_256v, // 101
// }

// /// Device operating mode
// enum Mode {
//     ContinuousConversion,
//     SingleShot,
// }

// enum DataRate {
//     _8SPS , // 000
//     _16SPS, // 001
//     _32SPS, // 010
//     _64SPS, // 011
//     _128SPS, // 100
//     _250SPS, // 101
//     _475SPS, // 110
//     _860SPS, // 111
// }

// enum CompMode {
//     Traditional,
//     Window
// }

// enum CompPol