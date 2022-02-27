// use linux_embedded_hal::I2cdev;
use linux_embedded_hal::I2cdev;
// use embedded_hal::i2c::blocking::{I2c};
// use linux_embedded_hal::i2cdev::core::I2CDevice;
use embedded_hal::blocking::i2c::{Write,Read};
use std::path::Path;

use crate::influx::Field;

pub struct SoilSensor {
    adc: ADS1115,
}


impl SoilSensor {
    pub fn new<P: AsRef<Path>>(path: P, address: u8) -> Self {
        let mut adc = ADS1115::new(path,address);
        adc.set_full_scale(Scale::FsPm4_096v);
        Self { adc }
    }

    pub fn measure(&mut self) -> [Field<f32>;1] {
        let m = self.adc.measure();
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
    scale: Scale,
}

const CONFIG_REG_ADDR : u8 = 1;
const CONVERSION_REG_ADDR : u8 = 0;

impl ADS1115 {
    pub fn new<P: AsRef<Path>>(path: P, address: u8) -> Self {
        println!(" fn");
        let i2c_device = I2cdev::new(path).unwrap();
        let conf_reg_msb = 0b01000100;
        let conf_reg_lsb = 0b10000011;
        let scale = Scale::FsPm2_048v; // default value
        let mut s = Self { i2c_device, address, conf_reg_msb, conf_reg_lsb, scale };
        s.update_config();
        s
    }

    fn measure(&mut self) -> f32 {
        println!("measure");
        let mut msb = 0;
        let mut lsb = 0;
        self.read_conv_reg(&mut msb, &mut lsb);
        let value = (msb as i16) << 8 | (lsb as i16);
        let f_value = value as f32;
        f_value*self.scale.get_scale()
    }
    fn set_full_scale(&mut self, scale: Scale) {
        // clear bits
        self.conf_reg_msb &= !0b00001110;
        self.conf_reg_msb |= scale.into_bits()<<1 ;
        self.scale = scale;
        self.update_config();
    }

    fn set_channel(&mut self, channel: u8) {
        assert!(channel < 4);
        // clear bits
        self.conf_reg_msb &= !0b01110000;
        self.conf_reg_msb |= 0b01000000 | (channel<<4) ;
        self.update_config();
    }

    fn update_config(&mut self) {
        self.write_to_config_reg(self.conf_reg_msb,self.conf_reg_lsb);
        self.check_config_reg();
    }

    fn check_config_reg(&mut self) {
        self.i2c_device.write(self.address, &[CONFIG_REG_ADDR]).unwrap();
        let mut data = [0u8; 2];
        self.i2c_device.read(self.address,&mut data).unwrap();
        let mut err = false;
        if self.conf_reg_msb != data[0] && (0b10000000^self.conf_reg_msb) != data[0] {
            err = true;
        }
        if self.conf_reg_lsb != data[1] {
            err = true;
        }
        if err {
            panic!("config not set properly.");
        }
    }

    fn write_to_config_reg(&mut self, data_msb: u8, data_lsb: u8) {
        // send address and aim pointer reg to conversion reg
        // then write data
        self.i2c_device.write(self.address, &[CONFIG_REG_ADDR,data_msb,data_lsb]).unwrap();
    }

    fn read_conv_reg(&mut self, data_msb: &mut u8, data_lsb: &mut u8) {
        // send address and aim pointer reg to conversion reg
        self.i2c_device.write(self.address, &[CONVERSION_REG_ADDR]).unwrap();
        // read 2 bytes
        let mut data = [0u8; 2];
        self.i2c_device.read(self.address,&mut data).unwrap();
        *data_msb = data[0];
        *data_lsb = data[1];
    }

}

#[allow(dead_code)]
/// Programmable gain amplifier configuration
enum Scale {
    FsPm6_144v, // 000
    FsPm4_096v, // 001
    FsPm2_048v, // 010
    FsPm1_024v, // 011
    FsPm0_512v, // 100
    FsPm0_256v, // 101
}

impl Scale {
    fn into_bits(&self)-> u8 {
        match self {
            Scale::FsPm6_144v => 0b000,
            Scale::FsPm4_096v => 0b001,
            Scale::FsPm2_048v => 0b010,
            Scale::FsPm1_024v => 0b011,
            Scale::FsPm0_512v => 0b100,
            Scale::FsPm0_256v => 0b101,
        }
    }

    fn get_scale(&self) -> f32 {
        match self {
            Scale::FsPm6_144v => 6.144/2.0f32.powi(15),
            Scale::FsPm4_096v => 4.096/2.0f32.powi(15),
            Scale::FsPm2_048v => 2.048/2.0f32.powi(15),
            Scale::FsPm1_024v => 1.024/2.0f32.powi(15),
            Scale::FsPm0_512v => 0.512/2.0f32.powi(15),
            Scale::FsPm0_256v => 0.256/2.0f32.powi(15),
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    #[ignore]
    fn get_adc_reading() {
        let mut ss = SoilSensor::new("/dev/i2c-1", 0x48);
        let measurements : Vec<f32> =  (0..4).into_iter().map(|_| {
            ss.adc.update_config();
            let reading = ss.measure();
            reading[0].value
        }).collect();
        println!("measurements: {measurements:?}");
        ss.adc.set_channel(0);
        ss.adc.update_config();
        println!("{:08b}-{:08b}",ss.adc.conf_reg_msb,ss.adc.conf_reg_lsb);
    }

    #[test]
    fn test_twos_compl_() {
        let u_a : u8 = 0b10011100;
        let u_b : u8 = 0b01010100;
        let i_c = (u_a as i16)<<8 | u_b as i16;
        println!("val: {i_c}");
        println!("bin: {i_c:016b}");
    }
}