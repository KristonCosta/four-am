use rusttype::Error as RTError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Font(RTError),
}

impl From<RTError> for Error {
    fn from(other: RTError) -> Self {
        Error::Font(other)
    }
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Error::Io(other)
    }
}
