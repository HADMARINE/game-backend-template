pub enum Event {
    Error,
    Refer,
    Execute,
    Terminate,

}

macro_rules! strm {
    ($x:expr) => {{
        let mut s = String::from($x);
        s
    }};
}

impl Event {
    pub fn to_string(&self) -> String {
        todo!()
        match *self {
            Event::Error => strm!("error"),
            Event::Refer => strm!("refer"),
        }
    }
}
