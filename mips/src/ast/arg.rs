use std::{fmt, fmt::Display};

use ast_traits::{AstNode, IntoAst};
use crate::ast::{
    Dev, DevBase, LineAbs, LineRel, MipsNode, Num, Reg, RegBase, RegLit,
};
use crate::{Alias, Aliases, MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Arg {
    Dev(Dev),
    Reg(Reg),
    Num(Num),
    LineAbs(LineAbs),
    LineRel(LineRel),
    String(String),
}

impl Arg {
    pub fn fixed(&self) -> bool {
        match self {
            Self::Reg(reg) => reg.fixed(),
            _ => false,
        }
    }

    pub fn set_fixed(&mut self, new_fixed: bool) {
        if let Self::Reg(reg) = self {
            reg.set_fixed(new_fixed);
        }
    }

    pub fn as_reg_lit(&self) -> Option<&RegLit> {
        match self {
            Self::Reg(reg) => reg.as_reg_lit(),
            _ => None,
        }
    }

    pub fn as_reg_lit_mut(&mut self) -> Option<&mut RegLit> {
        match self {
            Self::Reg(reg) => reg.as_reg_lit_mut(),
            _ => None,
        }
    }

    pub fn reduce(self, aliases: &Aliases) -> MipsResult<Self> {
        match self {
            Self::Dev(dev) => Ok(Self::Dev(dev.reduce(aliases)?)),
            Self::Reg(reg) => Ok(Self::Reg(reg.reduce(aliases)?)),
            Self::Num(num) => Ok(Self::Num(num.reduce(aliases)?)),
            Self::LineAbs(line_abs) => Ok(Self::LineAbs(line_abs.reduce(aliases)?)),
            Self::LineRel(line_rel) => Ok(Self::LineRel(line_rel.reduce(aliases)?)),
            Self::String(..) => Ok(self),
        }
    }
}

impl<'i> MipsNode<'i> for Arg {
    fn as_reg_base(&self) -> Option<RegBase> {
        match self {
            Self::Dev(dev) => dev.as_reg_base(),
            Self::Reg(reg) => reg.as_reg_base(),
            Self::Num(num) => num.as_reg_base(),
            Self::LineAbs(line_abs) => line_abs.as_reg_base(),
            Self::LineRel(line_rel) => line_rel.as_reg_base(),
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
            Self::String(..) => None,
        }
    }
}

impl From<RegBase> for Arg {
    fn from(reg_base: RegBase) -> Self {
        Self::Reg(reg_base.into())
    }
}

impl From<DevBase> for Arg {
    fn from(dev_base: DevBase) -> Self {
        Self::Dev(dev_base.into())
    }
}

impl From<&Alias> for Arg {
    fn from(alias: &Alias) -> Self {
        match alias {
            Alias::Num(n) => Self::Num(Num::Lit(*n)),
            Alias::Dev(dev_base) => Self::Dev(Dev::Base(*dev_base)),
            Alias::Reg(reg_base) => Self::Reg(Reg::Base(*reg_base)),
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
