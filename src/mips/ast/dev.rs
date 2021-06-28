use std::{fmt, fmt::Display};

use crate::ast_traits::{AstNode, AstPair, IntoAst};
use crate::mips::ast::{Arg, Num, MipsNode, RegLit, RegBase};
use crate::mips::{Mips, Alias, MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub struct DevLit {
    pub(crate) index: usize,
    pub(crate) indirections: usize,
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for DevLit {
    type Output = Self;

    const RULE: Rule = Rule::reg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::dev => {
                let indirections = pair.as_str().bytes().filter(|b| *b == b'r').count();
                let index = pair.only_inner()?.as_str().parse()?;
                Ok(Self { index, indirections })
            }
            _ => Err(MipsError::arg_wrong_kind("a literal device", pair)),
        }
    }
}

impl Display for DevLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "d")?;
        for _ in 0..self.indirections {
            write!(f, "r")?;
        }
        write!(f, "{}", self.index)
    }
}

#[derive(Clone, Debug)]
pub enum DevBase {
    DB,
    Lit(DevLit),
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for DevBase {
    type Output = Self;

    const RULE: Rule = Rule::reg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::dev => {
                let s = pair.as_str();
                match s {
                    "db" => Ok(Self::DB),
                    _ => Ok(Self::Lit(pair.try_into_ast()?)),
                }
            }
            _ => Err(MipsError::arg_wrong_kind("a base device", pair)),
        }
    }
}


impl Display for DevBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DB => write!(f, "db"),
            Self::Lit(t) => write!(f, "{}", t),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Dev {
    Base(DevBase),
    Alias(String),
}

impl MipsNode for Dev {
    fn as_reg_base(&self) -> Option<RegBase> {
        match self {
            &Self::Base(DevBase::Lit(DevLit { index, indirections })) if indirections > 0 => {
                Some(RegBase::Lit(RegLit { index, indirections: 0, fixed: true }))
            }
            _ => None,
        }
    }

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> {
        None
    }

    fn as_alias(&self) -> Option<&String> {
        match self {
            Self::Alias(key) => Some(key),
            _ => None,
        }
    }

    fn reduce(self, mips: &Mips) -> MipsResult<Self> {
        match self {
            Self::Base(..) => Ok(self),
            Self::Alias(key) => Ok(Self::Base(mips.get_only_dev_base(&key)?)),
        }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Dev {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::dev => Ok(Self::Base(pair.try_into_ast()?)),
            Rule::alias => Ok(Self::Alias(pair.as_str().to_owned())),
            _ => Err(MipsError::arg_wrong_kind("a device", pair)),
        }
    }
}

impl Display for Dev {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Base(t) => write!(f, "{}", t),
            Self::Alias(t) => write!(f, "{}", t),
        }
    }
}
