use linux_embedded_hal::{Delay, I2cdev};
use bme280::BME280 as BME280_extern;
use std::path::Path;
use crate::types::{Reading,Mapping, Sensor};

/// Creates wrapper around external BME280 implementation to
/// make interface similar to the moisture sensor
pub struct BME280 {
    dev: BME280_extern<I2cdev, Delay>,
    last_measurement: Option<[Mapping<'static,f32>;3]>
}

impl BME280 {
    /// Create new instance of BME280 sensor
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let i2c_bus = I2cdev::new(path).unwrap();
        let mut dev = BME280_extern::new_primary(i2c_bus, Delay);
        dev.init().unwrap();
        let last_measurement: Option<[Mapping<'static,f32>;3]> = None;
        Self { dev , last_measurement }
    }

}

impl Sensor for BME280 {
    type ReadingType = f32;
    /// Get Reading
    fn get_reading<'a>(&'a mut self) -> Reading<'a, 'static, f32> {
        let m = self.dev.measure().unwrap();
        let last_measurement = [
            Mapping{ key: "temperature" , value: m.temperature },
            Mapping{ key: "pressure"    , value: m.pressure    },
            Mapping{ key: "humidity"    , value: m.humidity    },
        ];
        self.last_measurement = Some(last_measurement);
        Reading { fields: self.last_measurement.as_ref().unwrap() }
    }
}