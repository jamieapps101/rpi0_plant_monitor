use std::time::Duration;

mod consts;

mod util;
use util::Event;

mod influx;
use influx::{DBConnection,Sample};

mod sensors;
use sensors::BME280;

use tokio::{sync::mpsc::channel,time::sleep};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // init sensor
    print!("Initialising Sensor... ");
    let mut sensor = BME280::new("/dev/i2c-1",0x76);
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
                    let reading_data = sensor.measure();
                    let s : Sample<'_, f32> = Sample {
                        measurement: "atmospherics",
                        tags:        &[("source","pzero").into(),("db_name","environmental").into()],
                        fields:      &reading_data,
                        time_stamp: None,
                    };

                    if let Err(reason) = client.send(s).await {
                        println!("Err: {reason}");
                        break;
                    }
                }
            }
        }

    };
}
