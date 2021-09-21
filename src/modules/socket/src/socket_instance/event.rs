pub enum ResponseEvent {
    Error,
    Redirect,
    Execute,
    Terminate,
    Ok,
    Data,
}

macro_rules! strm {
    ($x:expr) => {{
        let mut s = String::from($x);
        s
    }};
}

impl ResponseEvent {
    pub fn to_string(&self) -> String {
        match *self {
            ResponseEvent::Error => strm!("error"),
            ResponseEvent::Redirect => strm!("redirect"),
            ResponseEvent::Execute => strm!("execute"),
            ResponseEvent::Terminate => strm!("terminate"),
            ResponseEvent::Ok => strm!("ok"),
            ResponseEvent::Data => strm!("data"),
        }
    }
}
