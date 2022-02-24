use std::{thread::sleep, time::Duration};


mod influx;
use influx::{DBConnection,Sample};

use linux_embedded_hal::{Delay, I2cdev};
use bme280::BME280;


fn main() {
    // init sensor 
    print!("Initialising Sensor... ");
    let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();
    let mut sensor = BME280::new(i2c_bus,0x76,Delay);
    if let Err(reason) = sensor.init() {
        panic!("{:?}",reason);
    }
    println!("Done");
    
    // test connection to server
    loop {
        print!("Connecting to Server... ");
        let mut client;
        match DBConnection::new("tcp://192.168.1.224:30104","sensor_data") {
            Ok(c) => client = c,
            Err(reason) => {
                println!("\nErr: {:?}",reason);
                sleep(Duration::from_secs(10));
                continue;
            }
        }    
        println!("Done");
        
        // begin looping
        loop {
            let reading = sensor.measure().unwrap();
            let s : Sample<'_, f32> = Sample {
                measurement: "atmospherics",
                tags:        &[("source","pzero").into(),("db_name","environmental").into()],
                fields:      &[("temperature",reading.temperature).into(),
                ("preasure",reading.pressure).into(),
                ("humidity",reading.humidity).into()],
                time_stamp: None,
            };
            if let Err(reason) = client.send(s) {
                println!("Err: {:?}",reason);
                break;
            }
            sleep(Duration::from_secs(60));
        }
    }
}
