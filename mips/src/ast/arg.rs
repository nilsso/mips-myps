use std::{fmt, fmt::Display};

use ast_traits::{AstNode, IntoAst};
use crate::ast::{
    BatchMode, Dev, LineAbs, LineRel, MipsNode, Num, ReagentMode, Reg, RegBase,
};
use crate::{Mips, MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Arg {
    Dev(Dev),
    Reg(Reg),
    Num(Num),
    LineAbs(LineAbs),
    LineRel(LineRel),
    BatchMode(BatchMode),
    ReagentMode(ReagentMode),
    String(String),
}

impl MipsNode for Arg {
    fn as_reg_base(&self) -> Option<RegBase> {
        match self {
            Self::Dev(dev) => dev.as_reg_base(),
            Self::Reg(reg) => reg.as_reg_base(),
            Self::Num(num) => num.as_reg_base(),
            Self::LineAbs(line_abs) => line_abs.as_reg_base(),
            Self::LineRel(line_rel) => line_rel.as_reg_base(),
            Self::BatchMode(..) => None,
            Self::ReagentMode(..) => None,
            Self::String(..) => None,
        }
    }

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> {
        match self {
            Self::Dev(dev) => dev.as_reg_base_mut(),
            Self::Reg(reg) => reg.as_reg_base_mut(),
            Self::Num(num) => num.as_reg_base_mut(),
            Self::LineAbs(line_abs) => line_abs.as_reg_base_mut(),
            Self::LineRel(line_rel) => line_rel.as_reg_base_mut(),
            Self::BatchMode(..) => None,
            Self::ReagentMode(..) => None,
            Self::String(..) => None,
        }
    }

    fn as_alias(&self) -> Option<&String> {
        match self {
            Self::Dev(dev) => dev.as_alias(),
            Self::Reg(reg) => reg.as_alias(),
            Self::Num(num) => num.as_alias(),
            Self::LineAbs(line_abs) => line_abs.as_alias(),
            Self::LineRel(line_rel) => line_rel.as_alias(),
            Self::BatchMode(..) => None,
            Self::ReagentMode(..) => None,
            Self::String(..) => None,
        }
    }

    fn reduce(self, mips: &Mips) -> MipsResult<Self> {
        match self {
            Self::Dev(dev) => Ok(Self::Dev(dev.reduce(mips)?)),
            Self::Reg(reg) => Ok(Self::Reg(reg.reduce(mips)?)),
            Self::Num(num) => Ok(Self::Num(num.reduce(mips)?)),
            Self::LineAbs(line_abs) => Ok(Self::LineAbs(line_abs.reduce(mips)?)),
            Self::LineRel(line_rel) => Ok(Self::LineRel(line_rel.reduce(mips)?)),
            Self::BatchMode(batch_mode) => Ok(Self::BatchMode(batch_mode.reduce(mips)?)),
            Self::ReagentMode(reagent_mode) => Ok(Self::ReagentMode(reagent_mode.reduce(mips)?)),
            Self::String(..) => Ok(self),
        }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Arg {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::dev => Ok(Self::Dev(pair.try_into_ast()?)),
            Rule::reg => Ok(Self::Reg(pair.try_into_ast()?)),
            Rule::num => Ok(Self::Num(pair.try_into_ast()?)),
            Rule::alias => Ok(Self::String(pair.as_str().into())),
            _ => panic!("{:?}", pair),
        }
    }
}

macro_rules! impl_into_arg {
    ($(($kind:ty, $variant:path)),*$(,)*) => {
        $(
            impl From<$kind> for Arg {
                fn from(thing: $kind) -> Self {
                    $variant(thing)
                }
            }
        )*
    }
}

impl_into_arg!(
    (Dev, Arg::Dev),
    (Reg, Arg::Reg),
    (Num, Arg::Num),
    (LineAbs, Arg::LineAbs),
    (LineRel, Arg::LineRel),
    (BatchMode, Arg::BatchMode),
    (ReagentMode, Arg::ReagentMode),
    (String, Arg::String),
);

impl Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Dev(t) => write!(f, "{}", t),
            Self::Reg(t) => write!(f, "{}", t),
            Self::Num(t) => write!(f, "{}", t),
            Self::LineAbs(t) => write!(f, "{}", t),
            Self::LineRel(t) => write!(f, "{}", t),
            Self::BatchMode(t) => write!(f, "{}", t),
            Self::ReagentMode(t) => write!(f, "{}", t),
            Self::String(t) => write!(f, "{}", t),
        }
    }
}

pub struct DevOrReg;

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for DevOrReg {
    type Output = Arg;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::dev => Ok(Arg::Dev(pair.try_into_ast()?)),
            Rule::reg => Ok(Arg::Reg(pair.try_into_ast()?)),
            _ => Err(MipsError::pair_wrong_rule("a device or register", pair)),
        }
    }
}
