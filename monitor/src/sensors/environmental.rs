use linux_embedded_hal::{Delay, I2cdev};
use bme280::BME280 as Bme280Ext;

use std::path::Path;

use crate::influx::Field;

pub struct BME280 {
    sensor: bme280::BME280<linux_embedded_hal::I2cdev, linux_embedded_hal::Delay>,
}

impl BME280 {
    pub fn new<P: AsRef<Path>>(path: P, address: u8) -> Self {
        let i2c_bus = I2cdev::new(path).unwrap();
        let mut sensor = Bme280Ext::new(i2c_bus,address,Delay);
        if let Err(reason) = sensor.init() {
            panic!("{:?}",reason);
        }
        Self { sensor }
    }

    pub fn measure(&mut self) -> [Field<f32>;3] {
        let reading = self.sensor.measure().unwrap();
        [("temperature",reading.temperature).into(),
            ("pressure",reading.pressure).into(),
            ("humidity",reading.humidity).into()]
    }
}

