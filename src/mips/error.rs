use std::io::Error as IOError;
use std::num::{ParseFloatError, ParseIntError};
use std::{fmt, fmt::Display};

use pest::iterators::Pair;

use crate::ast_traits::{AstError, AstErrorBase};
use crate::mips::{Alias, Rule};

type PegError = pest::error::Error<Rule>;

#[derive(Debug)]
pub enum MipsError {
    PegError(PegError),
    IOError(IOError),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),

    AstErrorBase(AstErrorBase),

    LineError(String),

    AliasUndefined(String),
    AliasWrongKind(String),
    InstrUnknown(String),
    ArgsWrongNum(String),
    ArgWrongKind(String),
}

// impl AstError for MipsError {
//     fn pairs_wrong_num(e: usize, f: usize) -> Self {
//         Self::PairWrongNum(format!("Expected {} pairs, found {}", e, f))
//     }
// }

impl MipsError {
    pub fn alias_undefine(key: &str) -> Self {
        Self::AliasUndefined(format!("Alias {} undefined", key))
    }

    pub fn alias_wrong_kind(expected: &'static str, found: &Alias) -> Self {
        Self::AliasWrongKind(format!("Expected {} alias, found {:?}", expected, found))
    }

    pub fn instr_unknown(key: &str) -> Self {
        Self::InstrUnknown(format!("Instruction {} unknown", key))
    }

    pub fn args_wrong_num(expected: usize, found: usize) -> Self {
        Self::ArgsWrongNum(format!("Expected {} arguments, found {}", expected, found))
    }

    pub fn arg_wrong_kind(expected: &'static str, found: Pair<Rule>) -> Self {
        Self::ArgWrongKind(format!(
            "Expected {} argument, found {} (a {:?})",
            expected,
            found.as_str(),
            found.as_rule()
        ))
    }
}

impl Display for MipsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PegError(e) => write!(f, "{:?}", e),
            Self::IOError(e) => write!(f, "{:?}", e),
            Self::ParseIntError(e) => write!(f, "{:?}", e),
            Self::ParseFloatError(e) => write!(f, "{:?}", e),

            Self::AstErrorBase(e) => write!(f, "{}", e),

            Self::LineError(s)
            | Self::AliasUndefined(s)
            | Self::AliasWrongKind(s)
            | Self::InstrUnknown(s)
            | Self::ArgsWrongNum(s)
            | Self::ArgWrongKind(s) => write!(f, "{}", s),
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
    IOError,
    AstErrorBase,
    ParseIntError,
    ParseFloatError,
);
