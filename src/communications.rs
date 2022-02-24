use crate::types::{DataPoint,Mapping};
use rumqttc::{AsyncClient,EventLoop,MqttOptions, QoS};

pub struct Connection {
    client: AsyncClient,
    event_loop: EventLoop,
    qos: QoS,
}

#[derive(Debug)]
pub enum ConnectionErr {
    SendError
}

impl Connection {
    pub fn new<S: Into<String>>(host: S, port: u16) -> Option<Self> {
        let opts = MqttOptions::new("0", host, port);
        // not sure what 10 is in this
        let (client,event_loop) = AsyncClient::new(opts,10);

        Some(Self {
            client,
            event_loop,
            qos: 0.into_QoS(),
        })
    }

    fn set_qos<I: IntoQoS>(mut self, qos: I) -> Self {
        self.qos = qos.into_QoS(); self
    }

    pub async fn send<'a, D: Into<DataPoint<'a,String>>>(&self, topic: &str, data_source: D) -> Result<(),ConnectionErr> {
        let data_point: DataPoint<String> = data_source.into();
        self.client.publish(topic, self.qos, false, data_point).await.or(Err(ConnectionErr::SendError))
    }
}

trait IntoQoS {
    fn into_QoS(self) -> QoS;
}

impl IntoQoS for QoS {
    fn into_QoS(self) -> QoS { self }
}

impl IntoQoS for usize {
    fn into_QoS(self) -> QoS {
        match self {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => panic!("not a valid conversion"),
        }
     }
}




impl<'a>  Into<Vec<u8>> for DataPoint<'a,String> {
    fn into(self) -> Vec<u8> {
        // calculate string length
        let mut chars_required = 0;
        chars_required += self.measurement.len();
        if self.tag_set.len() > 0 {
            // for the commas
            chars_required+=self.tag_set.len();
            // for the tags
            let lens_iterator = self.tag_set.iter().map(|tags| tags.get_required_chars() );
            let tags_len: usize = Iterator::sum(lens_iterator);
            chars_required += tags_len;
        }
        if self.field_set.len() > 0 {
            // for the space and the commas
            chars_required+=self.field_set.len();
            // for the fields
            let lens_iterator = self.field_set.iter().map(|tags| tags.get_required_chars() );
            let tags_len: usize = Iterator::sum(lens_iterator);
            chars_required += tags_len;
        }
        let time_stamp_string = if let Some(time_stamp) = self.time_stamp {
            let ts_string = format!(" {time_stamp}");
            chars_required += ts_string.len();
            ts_string
        } else {
            String::default()
        };
        let mut line = String::with_capacity(chars_required);
        line.push_str(self.measurement);
        self.tag_set.iter().for_each(|tag| {
            tag.append_to_string(&mut line, true);
        });
        if self.field_set.len()>0 {
            line.push_str(" ");
            self.field_set.iter().enumerate().for_each(|(index,field)| {
                field.append_to_string(&mut line,index!=0);
            });
        }
        line.push_str(time_stamp_string.as_str());
        line.into_bytes()
    }
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn mapping_to_string() {
        let key = "plant";
        let value = "parsley";
        let m = Mapping {key , value};
        let ref_string = "plant=parsley,plant=parsley".to_owned();
        let mut string = String::new();
        m.append_to_string(&mut string, false);
        m.append_to_string(&mut string, true);
        assert_eq!(string,ref_string);
    }

    #[test]
    fn datapoint_to_vec() {
        let ref_line = "myMeasurement,tag1=value1,tag2=value2 fieldKey=fieldValue 1556813561098000000".to_owned();
        let tags = [
            Mapping {key: "tag1", value: "value1"},
            Mapping {key: "tag2", value: "value2"},
        ];
        let fields = [
            Mapping {key: "fieldKey", value: "fieldValue".to_owned()},
        ];
        let dp = DataPoint {
            measurement: "myMeasurement",
            tag_set: &tags[..],
            field_set: &fields[..],
            time_stamp: Some(1556813561098000000)
        };

        let line_bytes: Vec<u8> = dp.into();
        let line= String::from_utf8(line_bytes).unwrap();

        assert_eq!(ref_line,line)

    }
}
