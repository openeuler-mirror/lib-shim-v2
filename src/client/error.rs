use std::fmt;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    WithoutInit(String),
    InvalidArgument(String),
    ShimError(String),
    Other(String), 
    IOError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(ref s) => write!(f, "invalid argument:{}", s),
            Self::Other(ref s) => write!(f, "other error: {}", s),
            Self::WithoutInit(ref s) => write!(f, "connection has not been established: {}", s),
            Self::IOError(ref s) => write!(f, "io error: {}", s),
            Self::ShimError(ref s) => write!(f, "call shim-v2 failed: {}", s)
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err.to_string())
    }
}

impl std::error::Error for Error {}

#[macro_export]
macro_rules! shim_error {
    ($e: ident, $s: expr) => {
        |$e| Error::ShimError($s.to_string() + &" ".to_string() + &$e.to_string())
    };
}

#[macro_export]
macro_rules! other {
    ($s: expr) => {
        Error::Other($s.to_string())
    };
}

#[macro_export]
macro_rules! other_error {
    ($e: ident, $s: expr) => {
        |$e| Error::Other($s.to_string() + &" ".to_string() + &$e.to_string())
    };
}

#[macro_export]
macro_rules! invalid_argument_error {
    ($s: expr) => {
        |$s| Error::InvalidArgument($s.to_string())
    };
}

#[macro_export]
macro_rules! print_err {
    ($e: ident, $s: expr) => {
        |$e| {
            error!("{}: {}", $s, &$e.to_string());
            $e
        }
    };
}