use std::{fmt, fmt::Display};

use crate::ast::{MipsNode, Reg, RegBase};
use crate::{Alias, Aliases, MipsError, MipsParser, MipsResult, Pair, Rule};
use ast_traits::{AstNode, IntoAst};

#[derive(Clone, Debug)]
pub enum Num {
    Lit(f64),
    Reg(RegBase),
    Alias(String),
}

impl Num {
    pub fn reduce(self, aliases: &Aliases) -> MipsResult<Self> {
        match self {
            Self::Lit(..) | Self::Reg(..) => Ok(self),
            Self::Alias(key) => {
                let alias = aliases.get(&key).ok_or(MipsError::alias_undefined(&key))?;
                match alias {
                    Alias::Num(n) => Ok(Self::Lit(*n)),
                    Alias::Reg(reg_base) => Ok(Self::Reg(reg_base.clone())),
                    Alias::Dev(..) => {
                        Err(MipsError::alias_wrong_kind("a number or register", alias))
                    }
                }
            }
        }
    }
}

impl<'i> MipsNode<'i> for Num {
    fn as_reg_base(&self) -> Option<RegBase> {
        match self {
            Self::Reg(reg_base) => Some(reg_base.clone()),
            _ => None,
        }
    }

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> {
        match self {
            Self::Reg(reg_base) => Some(reg_base),
            _ => None,
        }
    }

    fn as_alias(&self) -> Option<&String> {
        match self {
            Self::Alias(key) => Some(key),
            _ => None,
        }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Num {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self> {
        match pair.as_rule() {
            Rule::num => Ok(Self::Lit(pair.as_str().parse().unwrap())),
            Rule::reg => Ok(Num::Reg(pair.try_into_ast()?)),
            Rule::alias => Ok(Num::Alias(pair.as_str().into())),
            _ => Err(MipsError::pair_wrong_rule("a number", pair)),
        }
    }
}

impl From<f64> for Num {
    fn from(num: f64) -> Self {
        Self::Lit(num)
    }
}

impl From<Reg> for Num {
    fn from(reg: Reg) -> Self {
        match reg {
            Reg::Base(reg_base) => Self::Reg(reg_base),
            Reg::Alias { key, .. } => Self::Alias(key),
        }
    }
}

impl From<RegBase> for Num {
    fn from(reg_base: RegBase) -> Self {
        Self::Reg(reg_base)
    }
}

impl From<String> for Num {
    fn from(key: String) -> Self {
        Self::Alias(key)
    }
}

impl Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lit(t) => write!(f, "{}", t),
            Self::Reg(t) => write!(f, "{}", t),
            Self::Alias(t) => write!(f, "{}", t),
        }
    }
}

pub struct NumLit;

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for NumLit {
    type Output = Num;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Num> {
        match pair.as_rule() {
            Rule::num => pair.try_into_ast(),
            _ => Err(MipsError::pair_wrong_rule("a number literal", pair)),
        }
    }
}
