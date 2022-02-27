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
        // let m = m*(2.048/2.0f32.powi(15));
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
        // default value
        let scale = Scale::FsPm2_048v;
        // let conf_reg_msb = 0b10000100;
        // let conf_reg_lsb = 0b10000011;
        let mut s = Self { i2c_device, address, conf_reg_msb, conf_reg_lsb, scale };
        // s.set_full_scale(Scale::FsPm6_144v);
        // s.set_channel(1);
        s.update_config();
        println!("");
        s
    }

    fn measure(&mut self) -> f32 {
        println!("measure");
        let mut msb = 0;
        let mut lsb = 0;
        self.read_conv_reg(&mut msb, &mut lsb);
        println!("measurement: {:08b}-{:08b}",msb,lsb);
        let mut value = (msb as u16) << 8 | (lsb as u16);
        println!("value 2cp: {:016b}",value);

        // if the msb is one, the value is negative
        let mut is_neg = false;
        if 0x8000&value == 0x8000 {
            value ^= 0x8000;
            is_neg = true;
        }

        println!("value nom: {:016b}",value);
        println!("value real: {}",value);
        
        let mut f_value = value as f32;
        if is_neg {
            f_value *= -1.0;
        }
        println!("f_value: {}",f_value);

        println!("");
        let f_value_scaled = f_value*self.scale.get_scale();
        println!("f_value_scaled: {}",f_value_scaled);
        f_value_scaled
    }
    fn set_full_scale(&mut self, scale: Scale) {
        println!("set_full_scale");
        // clear bits
        self.conf_reg_msb &= !0b00001110;
        self.conf_reg_msb |= scale.into_bits()<<1 ;
        self.scale = scale;
        self.update_config();
        println!("");
    }

    fn set_channel(&mut self, channel: u8) {
        println!("set_channel");
        assert!(channel < 4);
        // clear bits
        println!("channel={channel}");
        println!("set_channel: A) {:08b}",self.conf_reg_msb);
        self.conf_reg_msb &= !0b01110000;
        println!("set_channel: B) {:08b}",self.conf_reg_msb);
        self.conf_reg_msb |= 0b01000000 | (channel<<4) ;
        println!("set_channel: C) {:08b}",self.conf_reg_msb);
        self.update_config();
        println!("");
    }

    fn update_config(&mut self) {
        println!("update_config");
        self.write_to_config_reg(self.conf_reg_msb,self.conf_reg_lsb);
        self.check_config_reg();
        println!("");
    }

    fn check_config_reg(&mut self) {
        println!("check_config_reg");
        self.i2c_device.write(self.address, &[CONFIG_REG_ADDR]).unwrap();
        let mut data = [0u8; 2];
        self.i2c_device.read(self.address,&mut data).unwrap();
        let mut err = false;
        if self.conf_reg_msb != data[0] && (0b10000000^self.conf_reg_msb) != data[0] {
            println!("self.conf_reg_msb: {:08b}",self.conf_reg_msb);
            println!("data[0]:           {:08b}",data[0]);
            err = true;
        }
        if self.conf_reg_lsb != data[1] {
            println!("self.conf_reg_lsb: {:08b}",self.conf_reg_lsb);
            println!("data[1]:           {:08b}",data[1]);
            err = true;
        }
        if err {
            panic!("config not set properly.");
        }
        println!("");
    }

    fn write_to_config_reg(&mut self, data_msb: u8, data_lsb: u8) {
        println!("write_to_config_reg");
        // send address and aim pointer reg to conversion reg
        // then write data
        // self.i2c_device.write(&[self.address,0b1,data_msb,data_lsb]).unwrap();
        self.i2c_device.write(self.address, &[CONFIG_REG_ADDR,data_msb,data_lsb]).unwrap();
        println!("");
    }

    fn read_conv_reg(&mut self, data_msb: &mut u8, data_lsb: &mut u8) {
        println!("read_conv_reg");
        // send address and aim pointer reg to conversion reg
        self.i2c_device.write(self.address, &[CONVERSION_REG_ADDR]).unwrap();
        // read 2 bytes
        let mut data = [0u8; 2];
        self.i2c_device.read(self.address,&mut data).unwrap();
        *data_msb = data[0];
        *data_lsb = data[1];
        println!("");
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
}