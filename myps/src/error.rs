use std::io::Error as IOError;
use std::num::{ParseFloatError, ParseIntError};
use std::{fmt, fmt::Display};

use ast_traits::AstErrorBase;
use crate::{Rule, Pair};

type PegError = pest::error::Error<Rule>;

#[derive(Debug)]
pub enum MypsError {
    PegError(PegError),
    IOError(IOError),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),

    AstErrorBase(AstErrorBase),

    Dummy,
}

impl Display for MypsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PegError(e) => write!(f, "{:?}", e),
            Self::IOError(e) => write!(f, "{:?}", e),
            Self::ParseIntError(e) => write!(f, "{:?}", e),
            Self::ParseFloatError(e) => write!(f, "{:?}", e),

            Self::AstErrorBase(e) => write!(f, "{}", e),

            Self::Dummy => write!(f, "dummy"),
        }
    }
}

macro_rules! impl_from_error {
    ($T:ty, $($E:tt),*$(,)*) => {
        $(
            impl From<$E> for $T {
                fn from(e: $E) -> Self {
                    <$T>::$E(e)
                }
            }
        )*
    }
}

impl_from_error!(
    MypsError,
    PegError,
    IOError,
    AstErrorBase,
    ParseIntError,
    ParseFloatError,
);

