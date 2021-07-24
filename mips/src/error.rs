use std::io::Error as IOError;
use std::num::{ParseFloatError, ParseIntError};
use std::{fmt, fmt::Display};

use ast_traits::AstErrorBase;
use crate::{Alias, Rule, Pair};

type PegError = pest::error::Error<Rule>;

#[derive(Debug)]
pub enum MipsError {
    PegError(PegError),
    IOError(IOError),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),

    AstErrorBase(AstErrorBase),

    LineError(String),

    IndexInvalid(String),
    AliasUndefined(String),
    AliasWrongKind(String),
    InstrUnknown(String),
    ArgsWrongNum(String),
    ArgsWrongKinds(String),
}

// impl AstError for MipsError {
//     fn pairs_wrong_num(e: usize, f: usize) -> Self {
//         Self::PairWrongNum(format!("Expected {} pairs, found {}", e, f))
//     }
// }

impl MipsError {
    pub fn index_invalid<T: std::fmt::Display>(index: T) -> Self {
        Self::IndexInvalid(format!("Index '{}' is invalid", index))
    }

    pub fn pair_wrong_rule<'i>(expected: &'static str, found: Pair<'i>) -> Self {
        Self::AstErrorBase(AstErrorBase::pair_wrong_rule(expected, found))
    }

    pub fn alias_undefined(key: &str) -> Self {
        Self::AliasUndefined(format!("Alias {} undefined", key))
    }

    pub fn alias_wrong_kind(expected: &'static str, found: &Alias) -> Self {
        Self::AliasWrongKind(format!("Expected {} alias, found {:?}", expected, found))
    }

    pub fn instr_unknown(key: &str) -> Self {
        Self::InstrUnknown(format!("Instruction {} unknown", key))
    }

    pub fn args_wrong_num(name: &str, expected: usize, found: usize) -> Self {
        Self::ArgsWrongNum(format!(
                "Instruction '{}' expects {} arguments, found {}",
                name,
                expected,
                found
        ))
    }

    pub fn args_wrong_kinds(
        name: &str,
        expected: &'static str,
        found: &str,
    ) -> Self {
        Self::ArgsWrongKinds(format!(
            "Instruction '{}' expects arguments ({}), found ({})",
            name,
            expected,
            found,
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

            Self::IndexInvalid(s)
            | Self::LineError(s)
            | Self::AliasUndefined(s)
            | Self::AliasWrongKind(s)
            | Self::InstrUnknown(s)
            | Self::ArgsWrongNum(s)
            | Self::ArgsWrongKinds(s) => write!(f, "{}", s),
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
