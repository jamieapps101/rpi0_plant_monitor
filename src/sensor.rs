extern crate i2cdev;
use i2cdev::{
    linux::*,
    core::I2CDevice,
    core::I2CTransfer,
};

pub struct BME280<T: I2CDevice + Sized> {
    dev: T,
    p_sample_rate: BME280SampleRate,
    // p_resolution:  f64,
    p_comp_vals:   [f64; 9],
    t_sample_rate: BME280SampleRate,
    // t_resolution:  f64,
    t_comp_vals:   [f64; 3],
    h_sample_rate: BME280SampleRate,
    // h_resolution:  f64,
    h_comp_vals:   [f64; 6],
    power_mode:    BME280PowerMode,
    filtering:     BME280FilterCoefs,
}

#[allow(dead_code)]
pub enum BME280Addr {
    LowAddr, 
    HighAddr,
}

impl From<BME280Addr> for u16 {
    fn from(addr: BME280Addr) -> Self {
        match addr {
            BME280Addr::LowAddr  => 0x76,
            BME280Addr::HighAddr => 0x77,
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum BME280Error {
    InvalidId,
    I2cError,
    ValueNotSet,
}

#[derive(Debug)]
pub struct BME280Reading {
    pub temp:         f64,
    pub preasure:     f64,
    pub humidity:     f64,
    raw_temp:     u32,
    raw_preasure: u32,
    raw_humidity: u32,
}

#[derive(Copy, Clone)]
pub enum BME280PowerMode {
    Sleep,
    ForcedMode,
    NormalMode,
}

impl From<BME280PowerMode> for u8 {
    fn from(other: BME280PowerMode) -> u8 {
        match other {
            BME280PowerMode::Sleep      => 0b00,
            BME280PowerMode::ForcedMode => 0b01,
            BME280PowerMode::NormalMode => 0b11,
        }
    }
}

#[derive(Copy, Clone)]
pub enum BME280SampleRate {
    Skip,
    X1,
    X2,
    X4,
    X8,
    X16,
}

impl From<BME280SampleRate> for u8 {
    fn from(other: BME280SampleRate) -> u8 {
        match other {
            BME280SampleRate::Skip => 0b000,
            BME280SampleRate::X1 =>   0b001,
            BME280SampleRate::X2 =>   0b010,
            BME280SampleRate::X4 =>   0b011,
            BME280SampleRate::X8 =>   0b100,
            BME280SampleRate::X16 =>  0b101,
        }
    }
}

#[derive(Clone,Copy)]
pub enum BME280FilterCoefs {
    Off,
    X2,
    X4,
    X8,
    X16,
}

impl From<BME280FilterCoefs> for u8 {
    fn from(other: BME280FilterCoefs) -> u8 {
        match other {
            BME280FilterCoefs::Off => 0b000,
            BME280FilterCoefs::X2  => 0b001,
            BME280FilterCoefs::X4  => 0b010,
            BME280FilterCoefs::X8  => 0b011,
            BME280FilterCoefs::X16 => 0b100,
        }
    }
}

#[allow(dead_code)]
const HUMIDITY_0_REG      : u8 = 0xFE; // lsb
#[allow(dead_code)]
const HUMIDITY_1_REG      : u8 = 0xFD;
#[allow(dead_code)]
const HUMIDITY_2_REG      : u8 = 0xFC; // msb
#[allow(dead_code)]
const TEMPERATURE_0_REG   : u8 = 0xFC; // lsb
#[allow(dead_code)]
const TEMPERATURE_1_REG   : u8 = 0xFB;
#[allow(dead_code)]
const TEMPERATURE_2_REG   : u8 = 0xFA; // msb
#[allow(dead_code)]
const PRESSURE_0_REG      : u8 = 0xF9; // lsb
#[allow(dead_code)]
const PRESSURE_1_REG      : u8 = 0xF8;
#[allow(dead_code)]
const PRESSURE_2_REG      : u8 = 0xF7; // msb

#[allow(dead_code)]
const CONFIG_REG          : u8 = 0xF5; // sets rate filter and interface options of the device, can only be written in sleep mode
const CONTROL_MEASURE_REG : u8 = 0xF4; // sets data aquisition options
#[allow(dead_code)]
const STATUS_REG          : u8 = 0xF3;
const CONTROL_HUM_REG     : u8 = 0xF2;
const CALIB_REG_1         : u8 = 0xE1; // to 0xF0
const RESET_REG           : u8 = 0xE0;
#[allow(dead_code)]
const ID_REG              : u8 = 0xD0;
const CALIB_REG_0         : u8 = 0x88; // to 0xA1


impl BME280<LinuxI2CDevice> {
    pub fn new<T: Into<u16>>(addr: T) -> Self {
        let device_name = "/dev/i2c-1";
        let dev = LinuxI2CDevice::new(device_name, addr.into()).unwrap();
        // these settings represent the power on state of the device
        let return_struct = Self {
            dev,
            p_sample_rate: BME280SampleRate::Skip,
            // p_resolution:  Self::get_p_res(&BME280SampleRate::Skip),
            p_comp_vals:   [0.0;9],
            t_sample_rate: BME280SampleRate::Skip,
            // t_resolution:  Self::get_t_res(&BME280SampleRate::Skip),
            t_comp_vals:   [0.0;3],

            h_sample_rate: BME280SampleRate::Skip,
            // h_resolution:  Self::get_t_res(&BME280SampleRate::Skip),
            h_comp_vals:   [0.0;6],
            
            power_mode:    BME280PowerMode::Sleep,
            filtering:     BME280FilterCoefs::Off,
        };

        return return_struct;
    }

    fn concat(msb: u8, lsb: u8) -> u32 {
        (msb as u32) << 8 | (lsb as u32)
    }

    // initialises the registers and obtains the compensation
    // parameter values
    pub fn init(&mut self) -> Result<(),BME280Error> {
        // push current values to reg
        if let Err(reason) = self.update_control() { return Err(reason)}
        if let Err(reason) = self.update_config() { return Err(reason)}

        // optain comp values
        let mut vals : [u8;(3+9)*2+1] = [0; (3+9)*2+1];
        self.dev.smbus_write_byte(CALIB_REG_0).unwrap();
        self.dev.read(&mut vals).unwrap();

        for i in 0..12 {
            match i {
                0..=2 => {
                    self.t_comp_vals[i] =   Self::concat(vals[2*i+1],vals[2*i]) as f64;
                },
                3..=11 => {
                    self.p_comp_vals[i-3] = Self::concat(vals[2*i+1],vals[2*i]) as f64;
                }
                _ => unreachable!(),
            }
        }
        self.h_comp_vals[0] = vals[(3+9)*2] as f64;
        // read in next data batch of comp values
        let mut vals : [u8;8] = [0; 8];
        self.dev.smbus_write_byte(CALIB_REG_1).unwrap();
        self.dev.read(&mut vals).unwrap();

        self.h_comp_vals[1] = ((vals[1] as u32) << 8 | (vals[0] as u32)) as f64;
        self.h_comp_vals[2] =   vals[2] as f64;
        self.h_comp_vals[3] = ((vals[3] as u32) << 4 | ((vals[4] as u32) & 0b00001111 ) ) as f64;

        self.h_comp_vals[4] = (((vals[4] as u32) & 0b00001111 ) | (vals[5] as u32) << 4 ) as f64;
        
        self.h_comp_vals[5] = vals[6] as f64;
        return Ok(());
    }

    pub fn set_temp_sample_rate(&mut self, s: BME280SampleRate) {
        self.t_sample_rate = s;
    }
    
    pub fn set_preasure_sample_rate(&mut self, s: BME280SampleRate) {
        self.p_sample_rate = s;
    }

    pub fn set_humidity_sample_rate(&mut self, s: BME280SampleRate) {
        self.h_sample_rate = s;
    }

    pub fn set_power_mode(&mut self, p: BME280PowerMode) {
        self.power_mode = p;
    }

    pub fn set_filtering(&mut self, f: BME280FilterCoefs) {
        self.filtering = f;
    }

    pub fn update_control(&mut self) -> Result<(),BME280Error> {
        // set ctrl_hum
        let ctrl_hum  : u8 = u8::from(self.h_sample_rate);
        // println!("setting humidity reg");
        if let Err(reason) = self.write_and_check(CONTROL_HUM_REG,ctrl_hum) {
            return Err(reason);
        }
        
        // set ctrl_meas
        let ctrl_meas : u8 = 
        u8::from(self.t_sample_rate) << 5 |
        u8::from(self.p_sample_rate) << 2 |
        u8::from(self.power_mode);
        
        // println!("setting temp preasure and power reg");
        if let Err(reason) = self.write_and_check(CONTROL_MEASURE_REG,ctrl_meas) {
            return Err(reason);
        }
        return Ok(());
    }

    fn write_and_check(&mut self, addr: u8, data:u8) -> Result<(),BME280Error> {
        let write_data = &[addr,data];
        if let Err(_) = self.dev.write(write_data) {
            return Err(BME280Error::I2cError);
        } 

        // now check the data
        let mut read_data : [u8; 1] = [0; 1];
        let mut msgs = [
            LinuxI2CMessage::write(&[addr]),
            LinuxI2CMessage::read(&mut read_data)
        ];
        if let Err(err_code) = self.dev.transfer(&mut msgs) {
            println!("reset: poor transfer: {}",err_code);
            return Err(BME280Error::I2cError);
        }
        // println!("set to: {:b}",read_data[0]);
        if data != read_data[0] {
            return Err(BME280Error::ValueNotSet);
        }
        Ok(())
    }
    
    pub fn update_config(&mut self) -> Result<(),BME280Error> {
        let config: u8 = u8::from(self.filtering) << 2;
        // println!("setting config to: {:b}",config);
        return self.write_and_check(CONFIG_REG,config);
    }

    #[allow(dead_code)]
    pub fn force_sample(&mut self) -> Result<(),BME280Error> {
        let ctrl_meas : u8 = 
            u8::from(self.t_sample_rate) << 5 |
            u8::from(self.p_sample_rate) << 2 |
            u8::from(BME280PowerMode::ForcedMode);
        return self.write_and_check(CONTROL_MEASURE_REG,ctrl_meas);
    }

    pub fn check_id(&mut self) -> Result<(),BME280Error> {
        self.check_id_against(0x60)
    }

    fn check_id_against(&mut self, ref_id: u8) -> Result<(),BME280Error> {
        if let Err(code) = self.dev.smbus_write_byte(ID_REG)  {
            println!("check_id: poor transfer: {}",code);
            return Err(BME280Error::I2cError)
        }

        // let mut read_data : u8;
        // let mut msgs = [LinuxI2CMessage::read(&mut read_data)];
        if let Ok(read_data) = self.dev.smbus_read_byte() {
            let return_id = read_data;
            if return_id==ref_id {
                return Ok(())
            } else {
                println!("I got: {:#X}",return_id);
                return Err(BME280Error::InvalidId)
            }
        } 
        return Err(BME280Error::I2cError);
    }

    pub fn reset(&mut self) -> Result<(),BME280Error> {
        let mut data : [u8; 1] = [0xB6; 1];
        let mut msgs = [
            LinuxI2CMessage::write(&[RESET_REG]),
            LinuxI2CMessage::write(&mut data)
        ];
        if let Err(err_code) = self.dev.transfer(&mut msgs) {
            println!("reset: poor transfer: {}",err_code);
            return Err(BME280Error::I2cError);
        } 
        Ok(())
    }

    pub fn read(&mut self) -> Result<BME280Reading,BME280Error> {
        // Data readout is done by starting a burst
        // read from 0xF7 to 0xFC. The data are read out in an unsigned 20-bit format both for pressure
        // and for temperature.

        let mut data : [u8; 8] = [0; 8];
        let mut msgs = [
            LinuxI2CMessage::write(&[PRESSURE_2_REG]), // the first address in the data slice
            LinuxI2CMessage::read(&mut data)
        ];
        if let Err(err_code) = self.dev.transfer(&mut msgs) {
            println!("reset: poor transfer: {}",err_code);
            return Err(BME280Error::I2cError);
        } 
        let raw_preasure : u32 = ((data[0] as u32) << 12) | ((data[1] as u32) << 4) | ((data[2] as u32) >> 4);
        let raw_temp     : u32 = ((data[3] as u32) << 12) | ((data[4] as u32) << 4) | ((data[5] as u32) >> 4);
        let raw_humidity : u32 = ((data[6] as u32) << 8)  | (data[7] as u32);

        let temp = self.raw_temp_to_actual(raw_temp);
        let preasure = self.raw_preasure_to_actual(raw_preasure, temp);
        let humidity = self.raw_humidity_to_actual(raw_humidity, temp);

        let reading = BME280Reading {
            temp,
            preasure,
            humidity,
            raw_temp,
            raw_preasure,
            raw_humidity
        };
        Ok(reading)
    }

    const T_FINE_ADJUST : i64 = 0;

    // taken from bosch data sheet:
    fn raw_temp_to_actual(&self, adc_t: u32) -> f64 {
        /////// Double Precision
        let f_t = adc_t as f64;
        let var1 : f64 = (f_t/16384.0 - self.t_comp_vals[0]/1024.0) * self.t_comp_vals[1];
        let var2 : f64 = ((f_t/131072.0 - self.t_comp_vals[0]/8192.0)*
                            (f_t/131072.0-self.t_comp_vals[0]/8192.0))*self.t_comp_vals[2];
        let t_fine = var1+var2;
        let t = t_fine/5120.0;
        return t;

        // Integer Precision
        // if adc_t == 0x800000 {
        //     return 0.0;
        // }
        // let adc_t = (adc_t >> 4) as i64;

        // let var1 = adc_t/8 - (self.t_comp_vals[0] as i64 * 2);
        // let var1 = (var1 * self.t_comp_vals[1] as i64) / 2048;
        // let var2 = (adc_t/16) - self.t_comp_vals[0] as i64;
        // let var2 = (((var2 * var2) / 4096) * self.t_comp_vals[2] as i64) / 16384;

        // let t_fine = var1 + var2 + Self::T_FINE_ADJUST;

        // let t : i64 = (t_fine * 5 + 128) / 256;

        // return t as f64 / 100.0;
    }

    // taken from bosch data sheet:
    fn raw_preasure_to_actual(&self, adc_p: u32, t: f64) -> f64 {
        ////// Double precision
        let f_p = adc_p as f64;
        let t_fine = t*5120.0;
        let var1 = (t_fine/2.0)-64000.0;
        let var2 = var1*var1*self.p_comp_vals[5]/32768.0;
        let var2 = var2+var1*self.p_comp_vals[4]*2.0;
        let var2 = (var2/4.0)+(self.p_comp_vals[3]*65536.0);
        let var1 = (self.p_comp_vals[2]*var1*var1 /524288.0+self.p_comp_vals[1]*var1)/524288.0;
        let var1 = (1.0+var1/32768.0)*self.p_comp_vals[0];
        if var1 == 0.0 {
            // this avoids div by zero later
            println!("avoiding div 0");
            return 0.0;
        }
        let p = 1048576.0 - f_p;
        let p = (p-(var2/4096.0))*6250.0/var1;
        let var1 = self.p_comp_vals[8]*p*p/2147483648.0;
        let var2 = p*self.p_comp_vals[7]/32768.0;
        let p = p + (var1+var2+self.p_comp_vals[6])/16.0;
        return p;

        // discrete precision
        // if adc_p == 0x800000 {
        //     return 0.0;
        // }
        // let t_fine = (((t*256.0)-128.0)/5.0 )as i64;
        // let adc_p = adc_p >> 4;

        // let var1 = t_fine - 128000;
        // let var2 = var1 * var1 * self.p_comp_vals[5] as i64;
        // let var2 = var2 + ((var1 * self.p_comp_vals[4] as i64) * 131072);
        // let var2 = var2 + ((self.p_comp_vals[3] as i64) * 34359738368);
        // let var1 = ((var1 * var1 * self.p_comp_vals[2] as i64) / 256) +
        //         ((var1 * (self.p_comp_vals[1] as i64) * 4096));
        // let var3 = 140737488355328; // TODO: check this <<<
        // let var1 = (var3 + var1) * (self.p_comp_vals[0] as i64) / 8589934592;

        // if var1 == 0 {
        //     return 0.0;
        // }

        // let var4 = 1048576 - (adc_p as i64);
        // let var4 = (((var4 * 2147483648) - var2) * 3125) / var1;
        // let var1 = ((self.p_comp_vals[8] as i128) * (var4 as i128 / 8192) * (var4 as i128 / 8192)) /
        //         33554432;
        // let var1 = var1 as i64;
        // let var2 = ((self.p_comp_vals[7] as i64) * var4) / 524288;
        // let var4 = ((var4 + var1 + var2) / 256) + ((self.p_comp_vals[6] as i64) * 16);

        // let p = var4 as f64 / 256.0;

        // return p;
    }

    fn raw_humidity_to_actual(&self, adc_h: u32, t: f64) -> f64 {
        // Double precision
        let f_h = adc_h as f64;
        let t_fine = t*5120.0;

        let var_h = t_fine-76800.0;
        let var_h = (f_h - (self.h_comp_vals[3]*64.0 + self.h_comp_vals[4]/16384.0 * var_h)) *
                        (self.h_comp_vals[1]/65536.0 * (1.0+self.h_comp_vals[5]/67108864.0*var_h *
                        (1.0+self.h_comp_vals[2]/67108864.0*var_h)));
        let var_h = var_h * (1.0-self.h_comp_vals[0]*var_h/524288.0);
        return var_h.min(100.0).max(0.0);

        // discrete precision
        // if adc_h == 0x8000 {
        //     return 0.0;
        // }

        // let adc_h = adc_h as i32;
        // let t_fine = (((t*256.0)-128.0)/5.0 )as i32;
    
        // let var1 = t_fine - 76800;
        // let var2 = adc_h * 16384;
        // let var3 = ( self.h_comp_vals[4] as i32 ) * 1048576;
        // let var4 = (self.h_comp_vals[4] as i32) * var1;
        // let var5 = (((var2 - var3) - var4) + 16384) / 32768;
        // let var2 = (var1 * (self.h_comp_vals[5] as i32)) / 1024;
        // let var3 = (var1 * (self.h_comp_vals[2] as i32)) / 2048;
        // let var4 = ((var2 * (var3 + 32768)) / 1024) + 2097152;
        // let var2 = ((var4 * (self.h_comp_vals[1] as i32)) + 8192) / 16384;
        // let var3 = var5 * var2;
        // let var4 = ((var3 / 32768) * (var3 / 32768)) / 128;
        // let var5 = var3 - ((var4 * (self.h_comp_vals[0] as i32)) / 16);
        // let var5 = if var5 < 0 { 0 } else {var5 };
        // let var5 = if var5 > 419430400 { 419430400 } else { var5 };
        // let h : i32 = var5 / 4096;
    
        // return h as f64 / 1024.0;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{thread::sleep, time::Duration};
    #[test]
    #[ignore]
    fn with_sensor_init() {
        let _sensor = BME280::new(BME280Addr::LowAddr);
    }
    
    #[test]
    #[ignore]
    fn with_sensor_check_id() {
        let mut sensor = BME280::new(BME280Addr::LowAddr);
        sensor.reset().unwrap();
        sleep(Duration::from_millis(1000));
        // bosch datasheet states id should be 0x58, but this 
        // appears not to be the case for this module
        assert_eq!(Ok(()),sensor.check_id_against(0x60));
    }

    #[test]
    #[ignore]
    fn with_sensor_output_no_sampling() {
        let mut sensor = BME280::new(BME280Addr::LowAddr);
        sensor.reset().unwrap();
        sleep(Duration::from_millis(1000));
        let reading = sensor.read().unwrap();
        assert_eq!(reading.raw_temp,0x80000);
        assert_eq!(reading.raw_preasure,0x80000);
    }

    #[test]
    #[ignore]
    fn with_sensor_output_with_sampling() {
        println!("\n\n");
        let mut sensor = BME280::new(BME280Addr::LowAddr);
        sensor.reset().unwrap();
        sleep(Duration::from_millis(10));
        sensor.set_preasure_sample_rate(BME280SampleRate::X1);
        sensor.set_temp_sample_rate(BME280SampleRate::X1);
        sensor.set_humidity_sample_rate(BME280SampleRate::X1);

        sensor.set_filtering(BME280FilterCoefs::Off);
        sensor.set_power_mode(BME280PowerMode::NormalMode);

        sensor.init().unwrap();
        sleep(Duration::from_millis(10));
        let reading = sensor.read().unwrap();
        println!("\np: {:?}", reading.raw_preasure);
        println!("p: {:?}pa", reading.preasure);
        println!("p: {:?}kpa\n", reading.preasure/1000.0);
        println!("t: {:?}", reading.raw_temp);
        println!("t: {:?} c\n", reading.temp);
        println!("h: {:?} ",  reading.raw_humidity);
        println!("h: {:?} %", reading.humidity);
    }
}




