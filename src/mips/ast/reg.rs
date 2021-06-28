use std::{fmt, fmt::Display};

use crate::ast_traits::{AstNode, AstPair, IntoAst};
use crate::mips::ast::{Arg, Num, MipsNode};
use crate::mips::{Mips, Alias, MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub struct RegLit {
    pub(crate) index: usize,
    pub(crate) indirections: usize,
    pub(crate) fixed: bool,
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for RegLit {
    type Output = Self;

    const RULE: Rule = Rule::reg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::reg => {
                let indirections = pair.as_str().bytes().filter(|b| *b == b'r').count() - 1;
                let index = pair.only_inner()?.as_str().parse()?;
                Ok(Self { index, indirections, fixed: false })
            }
            _ => Err(MipsError::arg_wrong_kind("a literal register", pair)),
        }
    }
}

impl Display for RegLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..(self.indirections + 1) {
            write!(f, "r")?;
        }
        write!(f, "{}", self.index)
    }
}

#[derive(Clone, Debug)]
pub enum RegBase {
    SP,
    RA,
    Lit(RegLit),
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for RegBase {
    type Output = Self;

    const RULE: Rule = Rule::reg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::reg => {
                let s = pair.as_str();
                match s {
                    "sp" => Ok(Self::SP),
                    "ra" => Ok(Self::RA),
                    _ => Ok(Self::Lit(pair.try_into_ast()?)),
                }
            }
            _ => Err(MipsError::arg_wrong_kind("a base register", pair)),
        }
    }
}

impl Display for RegBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SP => write!(f, "sp"),
            Self::RA => write!(f, "ra"),
            Self::Lit(t) => write!(f, "{}", t),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Reg {
    Base(RegBase),
    Alias(String),
}

impl MipsNode for Reg {
    fn as_reg_base(&self) -> Option<RegBase> {
        match self {
            Self::Base(reg_base) => Some(reg_base.clone()),
            _ => None,
        }
    }

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> {
        match self {
            Self::Base(reg_base) => Some(reg_base),
            _ => None,
        }
    }

    fn as_alias(&self) -> Option<&String> {
        match self {
            Self::Alias(key) => Some(key),
            _ => None,
        }
    }

    fn reduce(self, mips: &Mips) -> MipsResult<Self> {
        match self {
            Self::Alias(key) => Ok(Self::Base(mips.get_only_reg_base(&key)?)),
            _ => Ok(self),
        }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Reg {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::reg => Ok(Self::Base(pair.try_into_ast()?)),
            Rule::alias => Ok(Self::Alias(pair.as_str().to_owned())),
            _ => Err(MipsError::arg_wrong_kind("a register", pair)),
        }
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Base(t) => write!(f, "{}", t),
            Self::Alias(t) => write!(f, "{}", t),
        }
    }
}
