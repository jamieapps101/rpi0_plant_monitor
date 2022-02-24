pub struct Reading<'a,'b,T> {
    pub fields: &'a [Mapping<'b,T>]
}

pub trait Sensor {
    type ReadingType;
    fn get_reading<'a>(&'a mut self) -> Reading<'a, 'static, Self::ReadingType>;
}

pub struct DataPoint<'a,T:std::fmt::Display> {
    pub measurement: &'a str,
    pub tag_set: &'a[Mapping<'a,&'a str>],
    pub field_set: &'a [Mapping<'a,T>],
    pub time_stamp: Option<u64>
}

pub struct Mapping<'a,T> {
    pub key: &'a str,
    pub value: T,
}

impl<'a> Mapping<'a,&str> {
    pub fn get_required_chars(&self) -> usize {
        self.key.len()+self.value.len()+1
    }

    pub fn append_to_string(&self, s: &mut String,with_comma: bool) {
        if with_comma { s.push(',') }
        s.push_str(self.key);
        s.push('=');
        s.push_str(self.value);
    }
}

impl<'a> Mapping<'a,String> {
    pub fn get_required_chars(&self) -> usize {
        self.key.len()+self.value.len()+1
    }

    pub fn append_to_string(&self, s: &mut String,with_comma: bool) {
        if with_comma { s.push(',') }
        s.push_str(self.key);
        s.push('=');
        s.push_str(self.value.as_str());
    }
}


impl<'a,T> From<(&'a str,T)> for Mapping<'a,T> {
    fn from(d: (&'a str,T)) -> Self {
        Self { key: d.0, value: d.1 }
    }
}
