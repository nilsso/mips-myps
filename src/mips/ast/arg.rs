use std::{fmt, fmt::Display};

use crate::ast_traits::{AstNode, AstPair, IntoAst};
use crate::mips::ast::{Reg, Num, NumLit, LineAbs, LineRel};
use crate::mips::{MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Arg {
    Dev,
    Reg(Reg),
    Num(Num),
    LineAbs(LineAbs),
    LineRel(LineRel),
    String(String),
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Arg {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::arg => pair.only_inner()?.try_into_ast(),
            Rule::dev => panic!(),
            Rule::num => panic!(),
            Rule::alias => Ok(Self::String(pair.as_str().into())),
            _ => panic!("{:?}", pair),
        }
    }
}

impl From<Reg> for Arg {
    fn from(reg: Reg) -> Self {
        Self::Reg(reg)
    }
}

impl From<Num> for Arg {
    fn from(num: Num) -> Self {
        Self::Num(num)
    }
}

impl From<LineAbs> for Arg {
    fn from(line_abs: LineAbs) -> Self {
        Self::LineAbs(line_abs)
    }
}

impl From<LineRel> for Arg {
    fn from(line_rel: LineRel) -> Self {
        Self::LineRel(line_rel)
    }
}

impl From<String> for Arg {
    fn from(alias: String) -> Self {
        Self::String(alias)
    }
}

impl Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Dev => panic!(),
            Self::Reg(t) => write!(f, "{}", t),
            Self::Num(t) => write!(f, "{}", t),
            Self::LineAbs(t) => write!(f, "{}", t),
            Self::LineRel(t) => write!(f, "{}", t),
            Self::String(t) => write!(f, "{}", t),
        }
    }
}
