use std::{fmt, fmt::Display};

use ast_traits::AstNode;
use crate::ast::{MipsNode, RegBase};
use crate::{Mips, MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Copy, Clone, Debug)]
pub enum ModeRepr {
    Int,
    Str,
}

#[derive(Clone, Debug)]
pub enum BatchMode {
    Avg(ModeRepr),
    Sum(ModeRepr),
    Min(ModeRepr),
    Max(ModeRepr),
}

impl MipsNode for BatchMode {
    fn as_reg_base(&self) -> Option<RegBase> { None }
    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> { None }
    fn as_alias(&self) -> Option<&String> { None }
    fn reduce(self, _mips: &Mips) -> MipsResult<Self> {
        match self {
            Self::Avg(..) => Ok(Self::Avg(ModeRepr::Int)),
            Self::Sum(..) => Ok(Self::Sum(ModeRepr::Int)),
            Self::Min(..) => Ok(Self::Min(ModeRepr::Int)),
            Self::Max(..) => Ok(Self::Max(ModeRepr::Int)),
        }
    }
}

#[inline]
fn get_repr(int_s: &'static str, pair_s: &str) -> ModeRepr {
    if int_s == pair_s {
        ModeRepr::Int
    } else {
        ModeRepr::Str
    }
}

#[inline]
fn repr_disp(repr: &ModeRepr, int_repr: &'static str, str_repr: &'static str) -> &'static str {
    if matches!(repr, ModeRepr::Int) {
        int_repr
    } else {
        str_repr
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for BatchMode {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        let s = pair.as_str();
        #[rustfmt::skip]
        match s {
            "0" | "Average" => Ok(Self::Avg(get_repr("0", s))),
            "1" | "Sum"     => Ok(Self::Sum(get_repr("1", s))),
            "2" | "Minimum" => Ok(Self::Min(get_repr("2", s))),
            "3" | "Maximum" => Ok(Self::Max(get_repr("3", s))),
            _ => Err(MipsError::pair_wrong_rule("a batch mode", pair)),
        }
    }
}

impl Display for BatchMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Avg(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "0", "Average")),
            Self::Sum(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "1", "Sum")),
            Self::Min(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "2", "Min")),
            Self::Max(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "3", "Max")),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ReagentMode {
    Contents(ModeRepr),
    Required(ModeRepr),
    Recipe(ModeRepr),
}

impl MipsNode for ReagentMode {
    fn as_reg_base(&self) -> Option<RegBase> { None }
    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> { None }
    fn as_alias(&self) -> Option<&String> { None }
    fn reduce(self, _mips: &Mips) -> MipsResult<Self> {
        #[rustfmt::skip]
        match self {
            Self::Contents(..) => Ok(Self::Contents(ModeRepr::Int)),
            Self::Required(..) => Ok(Self::Required(ModeRepr::Int)),
            Self::Recipe(..)   => Ok(Self::Recipe(ModeRepr::Int)),
        }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for ReagentMode {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        let s = pair.as_str();
        #[rustfmt::skip]
        match pair.as_str() {
            "0" | "Contents" => Ok(Self::Contents(get_repr("0", s))),
            "1" | "Required" => Ok(Self::Required(get_repr("1", s))),
            "2" | "Recipe"   => Ok(Self::Recipe(get_repr("2", s))),
            _ => Err(MipsError::pair_wrong_rule("a reagent mode", pair)),
        }
    }
}

impl Display for ReagentMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Contents(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "0", "Contents")),
            Self::Required(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "1", "Required")),
            Self::Recipe(mode_repr)   => write!(f, "{}", repr_disp(mode_repr, "2", "Recipe")),
        }
    }
}

