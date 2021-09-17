use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct PortNotFoundError;

impl Error for PortNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot find vacant port")
    }
}
