use linux_embedded_hal::i2cdev::{core::*,linux::{LinuxI2CDevice,LinuxI2CError}};
use std::path::Path;
use crate::types::{Reading,Sensor};


pub struct MoistureSensor {
    i2c_dev: LinuxI2CDevice,
}


/// List of addresses
const CONVERSION_REG: u8  = 0x00;
const CONFIG_REG: u8      = 0x01;
const LOW_THRESH_REG: u8  = 0x10;
const HIGH_THRESH_REG: u8 = 0x11;


impl MoistureSensor {
    pub fn new<P: AsRef<Path>>(i2c_bus_path: P, addr: u16) -> Option<Self> {
        let i2c_dev = if let Ok(d) = LinuxI2CDevice::new(i2c_bus_path,addr) {
            d
        } else {
            return None;
        };
        let mut ms = Self{i2c_dev};

        if ms.init_device().is_err() {

        }

        Some(ms)
    }

    fn init_device(&mut self) -> Result<(),LinuxI2CError> {
        let config_reg_upper_byte: u8 = 0b00000000;
        let config_reg_lower_byte: u8 = 0b00000000;
        let config_reg = [config_reg_upper_byte,config_reg_lower_byte];
        self.set_config_reg_raw(config_reg)
    }

    fn read_conversion_reg(&mut self, values: &mut [u8;2]) -> Result<(),LinuxI2CError> {
        let data_result: Result<Vec<u8>, LinuxI2CError> = self.i2c_dev.smbus_read_block_data(CONVERSION_REG);
        match data_result {
            Ok(data) => {
                assert_eq!(data.len(),2);
                values[0] = data[0];
                values[1] = data[1];
                Ok(())
            },
            Err(reason) => {
                return Err(reason)
            }
        }
    }

    fn set_config_reg_raw(&mut self, values: [u8;2]) -> Result<(),LinuxI2CError> {
        self.i2c_dev.smbus_write_block_data(CONFIG_REG, &values)

    }
}

impl Sensor for MoistureSensor {
    type ReadingType = f32;
    /// Get Reading
    fn get_reading<'a>(&'a mut self) -> Reading<'a, 'static, f32> {
        unimplemented!();
    }
}
