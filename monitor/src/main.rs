use common::futures_util::StreamExt;
use common::serde_json;
use common::tokio;
use common::tokio::{
    sync::{mpsc::channel, Mutex},
    time::sleep,
};
use std::sync::Arc;
use std::time::Duration;

mod actuation;
mod config;
mod influx;
mod sensors;
mod util;

use actuation::{Command, Gpio, GpioAction};
use config::load;
use influx::{DBConnection, Sample};
use sensors::{SoilSensor, BME280};
use util::Event;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // load config toml, from somewhere.
    println!("Reading config...       ");
    let mut config: Option<config::Config> = None;
    let config_options = [
        "./monitor/config/config.toml",
        "/etc/plant_monitor/config.toml",
        "./monitor/config/config.toml.example",
    ];
    for config_path in config_options.iter() {
        if let Ok(c) = load(config_path) {
            println!("-> Reading from {config_path}");
            config = Some(c);
            break;
        }
    }
    let config = config.or_else(|| panic!("No config detected")).unwrap();
    println!("Done");

    // init sensor
    print!("Initialising Sensors... ");
    let mut env_sensor = BME280::new("/dev/i2c-1", 0x76);
    let mut soil_sensor = SoilSensor::new("/dev/i2c-1", 0x48);
    println!("Done");

    // init actuation
    let gpio_arcmut = Arc::new(Mutex::new(Gpio::new(config.hardware)));

    let (event_sink, mut event_source) = channel::<Event>(5);
    // create time management
    tokio::spawn(
        async move { util::ticker(event_sink, config.sampling.sample_period_seconds).await },
    );
    // test connection to server
    loop {
        print!("Connecting to Server... ");
        let (mut client, mut msg_stream) = match DBConnection::new(config.mqtt.clone()).await {
            Ok(c) => c,
            Err(reason) => {
                println!("\nErr:  {reason}");
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        };
        println!("Done");

        // setup async for receiving messages
        // accepts json commands, eg:
        // "{\"gpio\":1,\"state\":\"High\"}"
        let (gc_s, mut gc_r) = channel::<Command>(5);
        let gpio_arcmut_c = gpio_arcmut.clone();
        let gc_s_msg = gc_s.clone();
        tokio::spawn(async move {
            while let Some(msg_opt) = msg_stream.next().await {
                if let Some(msg) = msg_opt {
                    let message_content = std::str::from_utf8(msg.payload()).unwrap();

                    let command: Result<Command, serde_json::Error> =
                        serde_json::from_str(message_content);
                    match command {
                        Ok(command) => {
                            gc_s_msg.send(command).await.unwrap();
                        }
                        Err(reason) => println!("Unknown message: {message_content}\n({reason})"),
                    }
                }
            }
        });

        // gpio control loop
        tokio::spawn(async move {
            while let Some(command) = gc_r.recv().await {
                let gpio_mut = &*gpio_arcmut_c;
                let mut gpio = gpio_mut.lock().await;
                if let GpioAction::Pulse(period) = command.action {
                    let on_command = Command {
                        output: command.output,
                        action: GpioAction::On,
                    };
                    let off_command = Command {
                        output: command.output,
                        action: GpioAction::Off,
                    };
                    let gc_s_temp = gc_s.clone();
                    tokio::spawn(async move {
                        sleep(Duration::from_secs(period)).await;
                        gc_s_temp.send(off_command).await.unwrap()
                    });
                    gpio.set(on_command);
                } else {
                    gpio.set(command);
                }
            }
        });

        // begin looping
        let tags: Vec<influx::Tag> = config
            .sampling
            .tags
            .iter()
            .map(|t| (t.0.as_str(), t.1.as_str()).into())
            .collect();
        let measurement_name = config.sampling.measurement_name.as_str();
        while let Some(event) = event_source.recv().await {
            match event {
                Event::Tick => {
                    let env_reading_data = env_sensor.measure();
                    let mut s: Sample<'_, f32> = Sample {
                        measurement: measurement_name,
                        tags: &tags[..],
                        fields: &env_reading_data,
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
    }
}
