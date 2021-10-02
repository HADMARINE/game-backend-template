pub enum ResponseStatus {
    Error,
    Redirect,
    Execute,
    Terminate,
    Ok,
    Data,
}

macro_rules! strm {
    ($x:expr) => {{
        let s = String::from($x);
        s
    }};
}

impl ResponseStatus {
    pub fn to_string(&self) -> String {
        match *self {
            ResponseStatus::Error => strm!("error"),
            ResponseStatus::Redirect => strm!("redirect"),
            ResponseStatus::Execute => strm!("execute"),
            ResponseStatus::Terminate => strm!("terminate"),
            ResponseStatus::Ok => strm!("ok"),
            ResponseStatus::Data => strm!("data"),
        }
    }
}
