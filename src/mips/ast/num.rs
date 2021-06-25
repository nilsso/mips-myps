use std::{fmt, fmt::Display};

use crate::ast_traits::{AstNode, AstPair, IntoAst};
use crate::mips::ast::Reg;
use crate::mips::{MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Num {
    Lit(f64),
    Reg(Reg),
    Alias(String),
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Num {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self> {
        match pair.as_rule() {
            Rule::num => Ok(Self::Lit(pair.as_str().parse().unwrap())),
            Rule::reg => Ok(Self::Reg(pair.try_into_ast()?)),
            Rule::alias => Ok(Self::Alias(pair.as_str().to_owned())),
            _ => Err(MipsError::arg_wrong_kind("a number", pair)),
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
        Self::Reg(reg.into())
    }
}

impl From<String> for Num {
    fn from(alias: String) -> Self {
        Self::Alias(alias)
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
            _ => Err(MipsError::arg_wrong_kind("a number literal", pair)),
        }
    }
}
