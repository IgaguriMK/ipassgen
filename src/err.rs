use std::error;
use std::fmt;
use std::io;
use std::num;

#[derive(Debug)]
pub struct Error(Box<dyn error::Error>);

impl Error {
    pub fn new(msg: String) -> Error {
        Error(Box::new(StrError(msg)))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl error::Error for Error {}

macro_rules! impl_error_from {
    ($name: ty) => {
        impl From<$name> for Error {
            fn from(e: $name) -> Error {
                Error(Box::new(e))
            }
        }
    };
}

impl_error_from!(io::Error);
impl_error_from!(num::ParseFloatError);
impl_error_from!(num::ParseIntError);

impl_error_from!(rand::Error);

#[derive(Debug)]
struct StrError(String);

impl fmt::Display for StrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl error::Error for StrError {}

// #[derive(Debug)]
// pub struct WrapError(Box<dyn fmt::Debug>);

// impl fmt::Display for WrapError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         self.0.fmt(f)
//     }
// }

// impl error::Error for WrapError {}

// macro_rules! impl_error_from_wrap {
//     ($name: ty) => {
//         impl From<$name> for Error {
//             fn from(e: $name) -> Error {
//                 Error(Box::new(WrapError(Box::new(e))))
//             }
//         }
//     };
// }
