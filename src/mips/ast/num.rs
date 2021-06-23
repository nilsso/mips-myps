use std::{fmt, fmt::Display};
use std::boxed::Box;

use pest::iterators::Pair;

use crate::ast_traits::OnlyInner;
use crate::mips::ast::{Var, MipsNode, MipsResult, Reg};
use crate::mips::{Mips, MipsParser, Rule};

#[derive(Clone, Debug)]
pub enum Num {
    Lit(f64),
    Var(Var),
}

impl Num {
    pub fn reg(&self) -> Option<&Reg> {
        match self {
            Self::Lit(_) => None,
            Self::Var(var) => Some(var.reg()),
        }
    }

    pub fn lifetime(&self) -> Option<(usize, usize)> {
        match self {
            Self::Var(var) => Some(var.lifetime()),
            _ => None,
        }
    }

    pub fn update_lifetime(&mut self, s_opt: Option<usize>, e_opt: Option<usize>) {
        match self {
            Self::Var(r) => r.update_lifetime(s_opt, e_opt),
            _ => {},
        }
    }

    pub fn reduce(self) -> Self {
        match self {
            Num::Var(var) => Num::Var(var.reduce()),
            _ => self,
        }
    }
}

impl<'i> MipsNode<'i, Rule, MipsParser> for Num {
    type Output = Self;

    const RULE: Rule = Rule::num;

    fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> MipsResult<Self> {
        // println!("try num from {:?}", pair);
        match pair.as_rule() {
            Rule::var => Self::try_from_pair(mips, pair.only_inner()?),
            Rule::num => Ok(Self::Lit(pair.as_str().parse().unwrap())),
            // Rule::reg => Ok(Self::Var(pair.try_into_ast(mips).unwrap())),
            Rule::alias => mips.get_num(pair.as_str()),
            _ => panic!("{:?}", pair),
        }
    }
}

impl Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lit(n) => write!(f, "{}", n),
            Self::Var(v) => write!(f, "{}", v),
        }
    }
}
