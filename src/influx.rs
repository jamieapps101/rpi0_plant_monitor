use paho_mqtt::{AsyncClient, MessageBuilder, CreateOptionsBuilder};
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

impl<'a,T> ToString for Sample<'a,T> where T: std::fmt::Display {
    // this formats to the influx data format
    #[inline(always)]
    fn to_string(&self) -> String {
        let tag_string = self.tags.iter().map(|t| {
            t.to_string()
        }).collect::<Vec<String>>().join("");

        let field_string = if !self.fields.is_empty() {
            let mut temp  : String = self.fields[0].to_string_0th();
            temp += self.fields.iter().skip(1).map(|f| {
                f.to_string()
            }).collect::<Vec<String>>().join("").as_str();
            temp
        } else {
            String::from("")
        };

        if let Some(time) = self.time_stamp {
            format!("{}{}{} {}",self.measurement,tag_string,field_string,time)
        } else {
            format!("{}{}{}",self.measurement,tag_string,field_string)
        }
    }
}

pub struct SampleStack<'a,'b,T> where T: std::fmt::Display {
    pub samples: &'b[Sample<'a,T>],
}

impl<'a,'b,T>ToString for SampleStack<'a,'b,T> where T: std::fmt::Display {
    // this formats to the influx data format
    #[inline(always)]
    fn to_string(&self) -> String {
        self.samples.iter().map(|s| s.to_string()).collect::<Vec<String>>().join("\n")
    }
}

impl<'a> ToString for Tag<'a> {
    #[inline(always)]
    fn to_string(&self) -> String {
            format!(",{}={}",self.key,self.value)
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

pub struct DBConnection {
    topic: String,
    client: AsyncClient,
}

impl DBConnection {
    pub async fn new(host: &str, topic: &str, client_id: &str) -> Result<Self,paho_mqtt::Error> {
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(host)
            .client_id(client_id)
            .finalize();
        let mqtt_client = AsyncClient::new(create_opts).unwrap();
        mqtt_client.connect(None).await.unwrap();
        Ok(Self {
            topic: topic.to_owned(),
            client: mqtt_client,
        })
    }

    pub async fn send<T : std::fmt::Display> (&mut self, data: &Sample<'_,T> ) -> Result<(),paho_mqtt::Error> {
        let msg = MessageBuilder::new()
            .topic(self.topic.as_str())
            .payload(data.to_string())
            .qos(0)
            .finalize();
        self.client.publish(msg).await
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
        assert_eq!(format!("weather 1"),s.to_string());
    }

    #[test]
    fn no_tags_one_field() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[],
            fields:      &[("temperature","82").into()],
            time_stamp:  Some(1465839830100400200),
        };
        assert_eq!(format!("weather temperature=82 1465839830100400200"),s.to_string());
    }

    #[test]
    fn no_tags_multi_field() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[],
            fields:      &[("location","us-midwest").into(),("location","texas").into()],
            time_stamp:  Some(01),
        };
        assert_eq!(format!("weather location=us-midwest,location=texas 1"),s.to_string());
    }

    #[test]
    fn one_tag_no_fields() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[("location","us-midwest").into(),("season","summer").into()],
            fields:      &[],
            time_stamp:  Some(01),
        };
        assert_eq!(format!("weather,location=us-midwest,season=summer 1"),s.to_string());
    }

    #[test]
    fn multi_tag_no_fields() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[("temperature","82").into(),("humidity","43").into()],
            fields:      &[],
            time_stamp:  Some(01),
        };
        assert_eq!(format!("weather,temperature=82,humidity=43 1"),s.to_string());
    }

    #[test]
    fn one_tags_one_fields() {
        let s : Sample<'_, &str> = Sample {
            measurement: "weather",
            tags:        &[("location","us-midwest").into()],
            fields:      &[("temperature","82").into()],
            time_stamp:  Some(1465839830100400200),
        };
        assert_eq!(format!("weather,location=us-midwest temperature=82 1465839830100400200"),s.to_string());
    }
}