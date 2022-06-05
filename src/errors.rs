use std::error::Error;
use std::fmt::{
    Display,
    Formatter
};

#[derive(Debug)]
pub struct GICSError {
    message: String
}
pub type GICSResult<T> = Result<T, GICSError>;

impl GICSError {
    pub fn new(message: &str) -> Self {
        Self { message: message.into() }
    }
}

impl Display for GICSError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.message)
    }
}

// Implemented so that I don't have to do a ton of remapping of errors
impl std::convert::From<std::io::Error> for GICSError {
    fn from(ioerr: std::io::Error) -> Self {
        Self {
            message: ioerr.to_string()
        }
    }
}

impl Error for GICSError {}