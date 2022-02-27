use std::time::Duration;
use futures_util::StreamExt;
use tokio::{sync::mpsc::channel,time::sleep};
use std::sync::Arc;

mod util;
mod influx;
#[allow(dead_code)]
mod sensors;
#[allow(dead_code)]
mod actuation;
mod config;

use util::Event;
use influx::{DBConnection,Sample};
#[allow(unused_imports)]
use sensors::{BME280,SoilSensor};
use config::load;
use actuation::{Command,Gpio};


#[tokio::main(flavor = "current_thread")]
async fn main() {
    // load config toml, from somewhere.
    print!("Reading config...       ");
    let config =  load("./config/config.toml")
        .or_else(|_| load("/etc/plant_monitor/config.toml"))
        .or_else(|_| load("./config/config.toml.example")).unwrap();
    println!("Done");

    // init sensor
    print!("Initialising Sensors... ");
    let mut env_sensor  = BME280::new("/dev/i2c-1",0x76);
    let mut soil_sensor = SoilSensor::new("/dev/i2c-1", 0x48);
    println!("Done");

    // init actuation
    let gpio = Arc::new(Gpio::new());

    let (event_sink,mut event_source) = channel::<Event>(5);
    // create time management
    tokio::spawn(async move {
        util::ticker(event_sink,config.sampling.sample_period_seconds).await
    });
    // test connection to server
    loop {
        print!("Connecting to Server... ");
        let (mut client,mut msg_stream) = match DBConnection::new(config.mqtt.clone()).await {
            Ok(c) => c,
            Err(reason) => {
                println!("\nErr: {:?}",reason);
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        };
        println!("Done");

        // setup async for receiving messages
        // accepts json commands, eg:
        // "{\"gpio\":1,\"state\":\"High\"}"
        let gpio_c = gpio.clone();
        tokio::spawn(async move {
            while let Some(msg_opt) = msg_stream.next().await {
                if let Some(msg) = msg_opt {
                    let message_content = std::str::from_utf8(msg.payload()).unwrap();
                    println!("Got message: \"{message_content}\"");

                    let command : Result<Command, serde_json::Error> = serde_json::from_str(message_content);
                    match command {
                        Ok(command) => (*gpio_c).set(command),
                        Err(reason) => println!("Unknown message: {message_content}\n({reason})")
                    }
                }
            }
        });



        // begin looping
        let tags : Vec<influx::Tag> = config.sampling.tags.iter().map(|t| {
            (t.0.as_str(),t.1.as_str()).into()
        }).collect();
        let measurement_name = config.sampling.measurement_name.as_str();
        while let Some(event) = event_source.recv().await {
            match event {
                Event::Tick => {
                    let env_reading_data = env_sensor.measure();
                    let mut s : Sample<'_, f32> = Sample {
                        measurement: measurement_name,
                        tags:        &tags[..],
                        fields:      &env_reading_data,
                        time_stamp: None,
                    };

                    if let Err(reason) = client.send(&s).await {
                        println!("Err: {reason}");
                        break;
                    }

                    let soil_reading_data = soil_sensor.measure();
                    s.fields = &soil_reading_data;
                    if let Err(reason) = client.send(&s).await {
                        println!("Err: {reason}");
                        break;
                    }

                }
            }
        }

    };
}
