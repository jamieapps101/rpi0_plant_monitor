use serde_derive::Deserialize;
use std::{
    fs::File,
    io::{BufReader,prelude::*},
    path::Path
};

#[derive(Debug)]
pub enum ConfigLoadResult {
    Ok(Config),
    CouldNotOpenFile,
    CouldNotReadFile,
    ParseError(toml::de::Error),
}

impl ConfigLoadResult {
    pub fn unwrap(self) -> Config {
        if let Self::Ok(config) = self {
            config
        } else {
            panic!("Could not extract config: {self:?}");
        }
    }
}

pub fn load<P: AsRef<Path>>(path: P) -> ConfigLoadResult {
    let f = match File::open(path) {
        Ok(f) => f,
        Err(_reason) => {
            return ConfigLoadResult::CouldNotOpenFile;
        }
    };
    let mut reader = BufReader::new(f);
    let mut config_string = String::new();
    if reader.read_to_string(&mut config_string).is_err() {
        return ConfigLoadResult::CouldNotReadFile;
    }
    match toml::from_str(config_string.as_str()) {
        Ok(c) => ConfigLoadResult::Ok(c),
        Err(reason) => ConfigLoadResult::ParseError(reason),
    }
}


#[derive(Deserialize,PartialEq,Debug)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub sampling: SamplingConfig
}

#[derive(Deserialize,PartialEq,Debug,Clone)]
pub struct MqttConfig {
    pub server: String,
    pub client_id: String,
    pub topic: String,
    pub qos: u8,
}

#[derive(Deserialize,PartialEq,Debug)]
pub struct SamplingConfig {
    pub sample_period_seconds: u64,
    pub measurement_name: String,
    pub tags: Vec<(String,String)>
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn load_example_config() {
        let example_path = "config/config.toml.example";
        let loaded_config = load(example_path).unwrap();
        let ref_config = Config {
            mqtt: MqttConfig {
                server    : "tcp://server:1883".to_owned(),
                client_id : "client_0".to_owned(),
                topic     : "telegraf/topic/here".to_owned(),
                qos       : 0
            }, 
            sampling: SamplingConfig {
                sample_period_seconds : 60,
                measurement_name      : "atmospheric".to_owned(),
                tags                  : vec![("source".to_owned(),"pzero".to_owned())]
            }
        };
        assert_eq!(loaded_config,ref_config);
    }
}