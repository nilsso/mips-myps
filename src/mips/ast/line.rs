use std::{fmt, fmt::Display};

use crate::ast_traits::{AstNode, AstPair, IntoAst};
use crate::mips::ast::{Reg, Num};
use crate::mips::{MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum LineAbs {
    Lit(usize),
    Alias(String),
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for LineAbs {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self> {
        match pair.as_rule() {
            Rule::num => Ok(LineAbs::Lit(pair.as_str().parse()?)),
            Rule::alias => Ok(LineAbs::Alias(pair.as_str().to_owned())),
            _ => Err(MipsError::arg_wrong_kind("an absolute line number alias (or label)", pair))
        }
    }
}

impl Display for LineAbs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lit(t) => write!(f, "{}", t),
            Self::Alias(t) => write!(f, "{}", t),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LineRel {
    Lit(isize),
    Alias(String),
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for LineRel {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self> {
        match pair.as_rule() {
            Rule::num => Ok(Self::Lit(pair.as_str().parse()?)),
            Rule::alias => Ok(Self::Alias(pair.as_str().to_owned())),
            _ => Err(MipsError::arg_wrong_kind("a relative line number or alias", pair)),
        }
    }
}

impl Display for LineRel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lit(t) => write!(f, "{}", t),
            Self::Alias(t) => write!(f, "{}", t),
        }
    }
}
