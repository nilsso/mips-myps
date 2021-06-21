use std::{fmt, fmt::Display};
use std::boxed::Box;

use pest::iterators::Pair;

use crate::mips::ast::{Reg, AstNode, AstError, AstResult, IntoAst};
use crate::mips::{Mips, MipsParser, Rule, Alias};

#[derive(Clone, Debug)]
pub enum Num {
    Lit(f64),
    Reg(Reg),
    Alias(String, Box<Num>),
}

impl Num {
    pub fn update_lifetime(&mut self, s_opt: Option<usize>, e_opt: Option<usize>) {
        match self {
            Self::Reg(r) => r.update_lifetime(s_opt, e_opt),
            Self::Alias(_, r) => r.update_lifetime(s_opt, e_opt),
            _ => {},
        }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, AstError> for Num {
    type Output = Self;

    const RULE: Rule = Rule::num;

    fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> AstResult<Self::Output> {
        match pair.as_rule() {
            Rule::num => Ok(Self::Lit(pair.as_str().parse().unwrap())),
            Rule::reg => Ok(Self::Reg(pair.try_into_ast(mips).unwrap())),
            Rule::alias => {
                let alias = mips.get_alias(pair.as_str());
                let num = match alias {
                    Some(Alias::Num(num)) => num.clone(),
                    Some(Alias::Reg(reg)) => Num::Reg(reg.clone()),
                    _ => panic!("{:?}", alias),
                };
                Ok(Self::Alias(pair.as_str().into(), Box::new(num)))
            },
            _ => panic!("{:?}", pair),
        }
    }
}

impl Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lit(n) => write!(f, "{}", n),
            Self::Reg(r) => write!(f, "{}", r),
            Self::Alias(s, _) => write!(f, "{}", s),
        }
    }
}
