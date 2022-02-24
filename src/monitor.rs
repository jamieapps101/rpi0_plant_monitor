use crate::{sensors,types::{Sensor,DataPoint,Mapping}};
use crate::communications::{Connection};

use std::time::Duration;
use tokio::{sync::mpsc,time::sleep};

pub async fn begin<S: Into<String>>(mqtt_host: S, mqtt_port: u16) {
    let mut m = Monitor::new(mqtt_host,mqtt_port);

    let (s,r) = mpsc::channel::<()>(1);
    tokio::join!(Monitor::keep_time(s),m.local_watch(r));
}

pub struct Monitor {
    env_sense: sensors::BME280,
    // moisture_sense: sensors::MoistureSensor,
    connection: Connection,
}

impl Monitor {

    fn new<S: Into<String>>(mqtt_host: S, mqtt_port: u16) -> Self {
        Self {
            env_sense      : sensors::BME280::new("i2c-0"),
            // moisture_sense : sensors::MoistureSensor::new("i2c-0",0x51).unwrap(),
            connection     : Connection::new(mqtt_host, mqtt_port).unwrap(),
        }
    }

    async fn keep_time(sender: mpsc::Sender<()>) {
        loop {
            sender.send(()).await.unwrap();
            sleep(Duration::from_millis(1000)).await;
        }
    }

    async fn local_watch(&mut self, mut time_keeper: mpsc::Receiver<()>) {
        while let Some(_) = time_keeper.recv().await {
            println!("Sampling");
            // get reading from sensor
            let reading = self.env_sense.get_reading();

            let string_values: Vec<Mapping<'static, String>> = reading.fields.iter().map(|f| {
                Mapping { key: f.key, value: format!("{}",f.value)}
            }).collect();

            // create datapoint
            let tag_set: &[Mapping<'static,&str>] = &[ ("test","true").into() ];
            let dp = DataPoint {
                measurement: "environment",
                tag_set,
                field_set: &string_values[..],
                time_stamp: None,
            };
            self.connection.send("telegraf/test", dp).await.unwrap();
        }
    }
}
