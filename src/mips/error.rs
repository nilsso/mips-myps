use std::io::Error as IOError;
use std::num::{ParseFloatError, ParseIntError};
use std::{fmt, fmt::Display};

use crate::ast_traits::AstError;
use crate::mips::Rule;

type PegError = pest::error::Error<Rule>;

#[derive(Debug)]
pub enum MipsError {
    PegError(PegError),
    AstError(AstError),
    IOError(IOError),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),

    LineError(String),
    AliasUndefined(String),
    AliasWrongKind,

    ArgsWrongAmount(String),
    ArgWrongKind(String),
}

impl MipsError {
    pub fn alias_undefined(alias: &str) -> Self {
        Self::AliasUndefined(format!("Alias {} undefined", alias))
    }

    pub fn args_wrong_amount(e: usize, f: usize) -> Self {
        Self::ArgsWrongAmount(format!("Expected {} arguments, found {}", e, f))
    }

    pub fn arg_wrong_kind(expected: &'static str, found: &str, rule: Rule) -> Self {
        Self::ArgWrongKind(format!(
            "Expected {} argument, found {} (a {:?})",
            expected, found, rule
        ))
    }
}

impl Display for MipsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PegError(e) => write!(f, "{:?}", e),
            Self::AstError(e) => write!(f, "{:?}", e),
            Self::IOError(e) => write!(f, "{:?}", e),
            Self::ParseIntError(e) => write!(f, "{:?}", e),
            Self::ParseFloatError(e) => write!(f, "{:?}", e),

            Self::LineError(s)
            | Self::AliasUndefined(s)
            | Self::ArgsWrongAmount(s)
            | Self::ArgWrongKind(s) => write!(f, "{}", s),

            Self::AliasWrongKind => write!(f, "{:?}", self),
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
    MipsError,
    PegError,
    AstError,
    IOError,
    ParseIntError,
    ParseFloatError,
);
