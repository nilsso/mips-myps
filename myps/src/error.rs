use std::io::Error as IOError;
use std::num::{ParseFloatError, ParseIntError};
use std::{fmt, fmt::Display};

type PegError = pest::error::Error<Rule>;

use ast_traits::AstErrorBase;

use crate::ast::{Lv, Rv};
use crate::{Pair, Rule};

#[derive(Debug)]
pub enum MypsError {
    PegError(PegError),
    IOError(IOError),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),

    AstErrorBase(AstErrorBase),

    AliasUndefined(String),

    FuncUnknown(String),
    FuncArgsWrongNum(String),
    FuncArgsWrongKinds(String),

    LvReservedName(String),
    LvRvAsnWrongNum(String),
    LvRvAsnWrongLvForRvDev(String),

    Dummy,
}

impl MypsError {
    pub fn alias_undefined(key: &String) -> Self {
        Self::AliasUndefined(format!("Alias '{}' undefined", key))
    }

    pub fn func_unknown(name: &str) -> Self {
        Self::FuncUnknown(format!("Unknown function '{}'", name))
    }

    pub fn func_args_wrong_num(name: &str, expected: usize, found: usize) -> Self {
        Self::FuncArgsWrongNum(format!(
            "Expected {} arguments for '{}', found {}",
            expected, name, found
        ))
    }

    pub fn func_args_wrong_kinds(name: &str, expected: &'static str, found: &str) -> Self {
        Self::FuncArgsWrongKinds(format!(
            "Instruction '{}' expects arguments ({}), found ({})",
            name, expected, found,
        ))
    }

    pub fn lv_reserved_name(name: String) -> Self {
        Self::LvReservedName(format!("L-value name '{}' is reserved", name))
    }

    pub fn lv_rv_asn_wrong_num(num_lv: usize, num_expr: usize) -> Self {
        Self::LvRvAsnWrongNum(format!(
            "Cannot assign {} r-values to {} l-values",
            num_expr, num_lv
        ))
    }

    pub fn lv_rv_asn_wrong_lv_for_rv_dev(lv: &Lv, rv: &Rv) -> Self {
        Self::LvRvAsnWrongLvForRvDev(format!(
            concat!(
                "A device r-value can only be assigned to a variable l-value\n",
                "Found l-value: {:#?}\n",
                "Found r-value: {:#?}"
            ),
            lv, rv
        ))
    }
}

impl Display for MypsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PegError(e) => write!(f, "{:?}", e),
            Self::IOError(e) => write!(f, "{:?}", e),
            Self::ParseIntError(e) => write!(f, "{:?}", e),
            Self::ParseFloatError(e) => write!(f, "{:?}", e),

            Self::AstErrorBase(e) => write!(f, "{}", e),

            Self::FuncUnknown(s)
            | Self::FuncArgsWrongNum(s)
            | Self::FuncArgsWrongKinds(s)
            | Self::LvReservedName(s)
            | Self::AliasUndefined(s)
            | Self::LvRvAsnWrongNum(s)
            | Self::LvRvAsnWrongLvForRvDev(s) => write!(f, "{}", s),

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
