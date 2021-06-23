use std::{fmt, fmt::Display};

use pest::iterators::Pair;

use crate::ast_traits::OnlyInner;
use crate::mips::ast::{Num, MipsNode, MipsResult, IntoMipsNode, Var};
use crate::mips::{Mips, MipsParser, Rule};

#[derive(Clone, Debug)]
pub enum Arg {
    Dev,
    Var(Var),
    Num(Num),
    String(String),
}

impl Arg {
    pub fn lifetime(&self) -> Option<(usize, usize)> {
        match self {
            Self::Var(var) => Some(var.lifetime()),
            Self::Num(num) => num.lifetime(),
            _ => None,
        }
    }

    pub fn update_lifetime(&mut self, s_opt: Option<usize>, e_opt: Option<usize>) {
        match self {
            Self::Var(r) => r.update_lifetime(s_opt, e_opt),
            Self::Num(n) => n.update_lifetime(s_opt, e_opt),
            _ => {},
        }
    }

    pub fn reduce(self) -> Self {
        match self {
            Self::Var(var) => Self::Var(var.reduce()),
            Self::Num(num) => Self::Num(num.reduce()),
            _ => self,
        }
    }
}

impl<'i> MipsNode<'i, Rule, MipsParser> for Arg {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::arg => pair.only_inner().unwrap().try_into_ast(mips),
            Rule::dev => panic!(),
            Rule::num => panic!(),
            Rule::alias => Ok(Self::String(pair.as_str().into())),
            _ => panic!("{:?}", pair),
        }
    }
}

impl From<Var> for Arg {
    fn from(var: Var) -> Self {
        Self::Var(var)
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
            Self::Var(v) => write!(f, "{}", v),
            Self::Num(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
        }
    }
}
