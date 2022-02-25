use std::time::Duration;

mod consts;

mod util;
use util::Event;

mod influx;
use influx::{DBConnection,Sample};

use linux_embedded_hal::{Delay, I2cdev};
use bme280::BME280;

use tokio::{sync::mpsc::channel,time::sleep};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // init sensor
    print!("Initialising Sensor... ");
    let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();
    let mut sensor = BME280::new(i2c_bus,0x76,Delay);
    if let Err(reason) = sensor.init() {
        panic!("{:?}",reason);
    }
    println!("Done");

    let (event_sink,mut event_source) = channel::<Event>(5);
    // create time management
    tokio::spawn(async move {
        util::ticker(event_sink).await
    });
    // test connection to server
    loop {
        print!("Connecting to Server... ");
        let mut client;
        match DBConnection::new(consts::MQTT_SERVER,consts::MQTT_TOPIC,crate::consts::MQTT_CLIENT_ID).await {
            Ok(c) => client = c,
            Err(reason) => {
                println!("\nErr: {:?}",reason);
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        }
        println!("Done");


        // begin looping
        while let Some(event) = event_source.recv().await {
            match event {
                Event::Tick => {
                    let reading = sensor.measure().unwrap();
                    let s : Sample<'_, f32> = Sample {
                        measurement: "atmospherics",
                        tags:        &[("source","pzero").into(),("db_name","environmental").into()],
                        fields:      &[("temperature",reading.temperature).into(),
                        ("preasure",reading.pressure).into(),
                        ("humidity",reading.humidity).into()],
                        time_stamp: None,
                    };
                        // println!("Err: {:?}",reason);
                    if let Err(reason) = client.send(s).await {
                        println!("Err: {reason}");
                        break;
                    }
                }
            }
        }

    };
}
