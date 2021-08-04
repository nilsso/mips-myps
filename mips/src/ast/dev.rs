use std::{fmt, fmt::Display};

use crate::ast::{FixMode, MipsNode, RegBase, RegLit};
use crate::{Aliases, MipsError, MipsParser, MipsResult, Pair, Rule};
use ast_traits::{AstNode, AstPair, IntoAst};

#[derive(Copy, Clone, Debug)]
pub struct DevLit {
    pub index: usize,
    pub indirections: usize,
}

impl DevLit {
    pub fn new(index: usize, indirections: usize) -> Self {
        Self {
            index,
            indirections,
        }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for DevLit {
    type Output = Self;

    const RULE: Rule = Rule::reg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::dev => {
                let indirections = pair.as_str().bytes().filter(|b| *b == b'r').count();
                let index = pair.only_inner()?.as_str().parse()?;
                Ok(Self {
                    index,
                    indirections,
                })
            }
            _ => Err(MipsError::pair_wrong_rule("a literal device", pair)),
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

#[derive(Copy, Clone, Debug)]
pub enum DevBase {
    DB,
    Lit(DevLit),
}

impl From<DevLit> for DevBase {
    fn from(dev_lit: DevLit) -> Self {
        Self::Lit(dev_lit)
    }
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
            _ => Err(MipsError::pair_wrong_rule("a base device", pair)),
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

impl Dev {
    pub fn reduce(self, aliases: &Aliases) -> MipsResult<Self> {
        match self {
            Self::Base(..) => Ok(self),
            Self::Alias(key) => Ok(Self::Base(aliases.try_get_dev_base(&key)?)),
        }
    }
}

impl From<DevBase> for Dev {
    fn from(dev_base: DevBase) -> Self {
        Self::Base(dev_base)
    }
}

impl From<String> for Dev {
    fn from(key: String) -> Self {
        Self::Alias(key)
    }
}

impl From<DevLit> for Dev {
    fn from(dev_lit: DevLit) -> Self {
        Self::Base(dev_lit.into())
    }
}

impl<'i> MipsNode<'i> for Dev {
    fn as_reg_base(&self) -> Option<RegBase> {
        match self {
            &Self::Base(DevBase::Lit(DevLit {
                index,
                indirections,
            })) if indirections > 0 => {
                let reg_lit = RegLit {
                    index,
                    indirections,
                    fix_mode: FixMode::None,
                };
                Some(RegBase::Lit(reg_lit))
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
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Dev {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::dev => Ok(Self::Base(pair.try_into_ast()?)),
            Rule::alias => Ok(Self::Alias(pair.as_str().to_owned())),
            _ => Err(MipsError::pair_wrong_rule("a device", pair)),
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
