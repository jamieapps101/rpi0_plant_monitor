use rumqttc::{MqttOptions, AsyncClient,QoS};
use std::string::ToString;

////////////////////////////////
////////// Data Types //////////
////////////////////////////////
pub struct Tag<'a> {
    pub key:   &'a str,
    pub value: &'a str,
}

impl<'a> From<(&'a str,&'a str)> for Tag<'a> {
    #[inline(always)]
    fn from(val: (&'a str,&'a str)) -> Self {
        Tag {
            key: val.0,
            value: val.1,
        }
    }
}

impl<'a> ToString for Tag<'a> {
    #[inline(always)]
    fn to_string(&self) -> String {
            format!(",{}={}",self.key,self.value)
    }
}

pub struct Field<'a,T> where T: std::fmt::Display {
    pub key:   &'a str,
    pub value: T,
}

impl<'a,T> From<(&'a str,T)> for Field<'a,T>  where T: std::fmt::Display {
    #[inline(always)]
    fn from(val: (&'a str,T)) -> Self {
        Field {
            key: val.0,
            value: val.1,
        }
    }
}


pub struct Sample<'a,T> where T: std::fmt::Display {
    pub measurement: &'a str,
    pub tags:        &'a[Tag<'a>],
    pub fields:      &'a[Field<'a,T>],
    pub time_stamp:  Option<u64>,
}

impl<'a,T> Sample<'a,T> where T: std::fmt::Display {
    #[inline(always)]
    pub fn into_string(&self, target: &mut String) {
        // empty strings content, but maintain capacity
        target.clear();
        //
        target.push_str(self.measurement);
        self.tags.iter().for_each(|t| target.push_str(t.to_string().as_str()) );
        if !self.fields.is_empty() {
            target.push_str(self.fields[0].to_string_0th().as_str());
            self.fields.iter().skip(1).for_each(|f| {
                target.push_str(f.to_string().as_str());
            });
        }
        if let Some(time) = self.time_stamp {
            target.push_str(format!(" {}",time).as_str());
        }

    }
}


impl<'a,T> Field<'a,T> where T: std::fmt::Display {
    #[inline(always)]
    fn to_string_0th(&self) -> String {
        format!(" {}={}",self.key,self.value)
    }
}


impl<'a,T> std::fmt::Display for Field<'a,T> where T: std::fmt::Display {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ",{}={}",self.key,self.value)
    }
}




///////////////////////////////////////////
////////// Connection Management //////////
///////////////////////////////////////////
// #[derive(Debug)]
// pub enum DBConnectionError {
//     ClientCreationError(Error),
//     ClientConnectionError(Error),
//     MessageSendError(Error),
// }

pub struct DBConnection<'a> {
    topic: &'a str,
    client: AsyncClient,
}

impl<'a> DBConnection<'a> {
    pub fn new<T:Into<String>>(host: T, port: u16, topic: &'a str) -> Self {
        let mqttoptions = MqttOptions::new("rpi0", host, port);
        let (mqtt_client, _eventloop) = AsyncClient::new(mqttoptions, 10);
        Self { topic, client: mqtt_client }
    }

    pub async fn send<V: Into<Vec<u8>>>(&mut self, data: V ) -> Result<(),rumqttc::ClientError> {
        self.client.publish(self.topic,QoS::AtLeastOnce,false,data).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn no_tags_no_fields() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[],
            fields:      &[],
            time_stamp:  Some(1),
        };
        let ref_string = format!("weather 1");
        assert_eq!(ref_string,s.to_string());

        let mut buf_string = String::new();
        s.into_string(&mut buf_string);
        assert_eq!(ref_string,buf_string);
    }

    #[test]
    fn no_tags_one_field() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[],
            fields:      &[("temperature","82").into()],
            time_stamp:  Some(1465839830100400200),
        };
        let ref_string = format!("weather temperature=82 1465839830100400200");
        assert_eq!(ref_string,s.to_string());

        let mut buf_string = String::new();
        s.into_string(&mut buf_string);
        assert_eq!(ref_string,buf_string);
    }

    #[test]
    fn no_tags_multi_field() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[],
            fields:      &[("location","us-midwest").into(),("location","texas").into()],
            time_stamp:  Some(01),
        };
        let ref_string = format!("weather location=us-midwest,location=texas 1");
        assert_eq!(ref_string,s.to_string());

        let mut buf_string = String::new();
        s.into_string(&mut buf_string);
        assert_eq!(ref_string,buf_string);
    }

    #[test]
    fn one_tag_no_fields() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[("location","us-midwest").into(),("season","summer").into()],
            fields:      &[],
            time_stamp:  Some(01),
        };
        let ref_string = format!("weather,location=us-midwest,season=summer 1");
        assert_eq!(ref_string,s.to_string());

        let mut buf_string = String::new();
        s.into_string(&mut buf_string);
        assert_eq!(ref_string,buf_string);
    }

    #[test]
    fn multi_tag_no_fields() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[("temperature","82").into(),("humidity","43").into()],
            fields:      &[],
            time_stamp:  Some(01),
        };
        let ref_string = format!("weather,temperature=82,humidity=43 1");
        assert_eq!(ref_string,s.to_string());

        let mut buf_string = String::new();
        s.into_string(&mut buf_string);
        assert_eq!(ref_string,buf_string);
    }

    #[test]
    fn one_tags_one_fields() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[("location","us-midwest").into()],
            fields:      &[("temperature","82").into()],
            time_stamp:  Some(1465839830100400200),
        };
        let ref_string = format!("weather,location=us-midwest temperature=82 1465839830100400200");
        assert_eq!(ref_string,s.to_string());

        let mut buf_string = String::new();
        s.into_string(&mut buf_string);
        assert_eq!(ref_string,buf_string);
    }
}