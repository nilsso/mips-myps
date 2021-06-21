use std::{fmt, fmt::Display};

use pest::iterators::Pair;

use crate::ast_traits::OnlyInner;
use crate::mips::ast::{Reg, Num, AstNode, AstError, AstResult, IntoAst};
use crate::mips::{Mips, MipsParser, Rule};

#[derive(Clone, Debug)]
pub enum Arg {
    Dev,
    Reg(Reg),
    Num(Num),
    String(String),
}

impl Arg {
    pub fn as_reg(&self) -> Option<Reg> {
        match self {
            Self::Reg(reg) => Some(reg.clone()),
            _ => None,
        }
    }

    pub fn update_lifetime(&mut self, s_opt: Option<usize>, e_opt: Option<usize>) {
        match self {
            Self::Reg(r) => r.update_lifetime(s_opt, e_opt),
            Self::Num(n) => n.update_lifetime(s_opt, e_opt),
            _ => {},
        }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, AstError> for Arg {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> AstResult<Self::Output> {
        match pair.as_rule() {
            Rule::arg => pair.only_inner().unwrap().try_into_ast(mips),
            Rule::dev => panic!(),
            Rule::reg => Ok(Self::Reg(pair.try_into_ast(mips).unwrap())),
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
            Self::String(t) => write!(f, "{}", t),
        }
    }
}
