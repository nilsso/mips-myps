use std::{fmt, fmt::Display};

use pest::iterators::Pair;

use crate::ast_traits::OnlyInner;
use crate::mips::ast::{MipsNode, MipsResult, Reg, IntoMipsNode};
use crate::mips::{Mips, MipsParser, Rule};

#[derive(Clone, Debug)]
pub enum Var {
    Reg {
        indirections: usize,
        reg: Reg,
    },
    Alias {
        name: String,
        reg: Reg,
    },
}

impl Var {
    pub fn reg(&self) -> &Reg {
        match self {
            Self::Reg { reg, .. } => reg,
            Self::Alias { reg, .. } => reg,
        }
    }

    pub fn reduce(self) -> Self {
        if let Var::Alias { reg, .. } = self {
            Var::Reg { indirections: 0, reg }
        } else {
            self
        }
    }

    pub fn update_reg(&mut self, mips: &Mips) {
        match self {
            Var::Reg { reg, .. } | Var::Alias { reg, .. } => {
                *reg = mips
                    .aliases
                    .get(&reg.to_string())
                    .unwrap()
                    .reg()
                    .unwrap()
                    .clone();
            }
        }
    }

    pub fn lifetime(&self) -> (usize, usize) {
        match self {
            Self::Reg{ reg, .. } => reg.lifetime(),
            Self::Alias{ reg, .. } => reg.lifetime(),
        }
    }

    pub fn update_lifetime(&mut self, s_opt: Option<usize>, e_opt: Option<usize>) {
        match self {
            Self::Reg { reg, .. } => reg.update_lifetime(s_opt, e_opt),
            Self::Alias { reg, .. } => reg.update_lifetime(s_opt, e_opt),
        }
    }
}

impl<'i> MipsNode<'i, Rule, MipsParser> for Var {
    type Output = Self;

    const RULE: Rule = Rule::var;

    fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> MipsResult<Self> {
        // println!("try var from {:?}", pair);
        match pair.as_rule() {
            Rule::var => pair.only_inner()?.try_into_ast(mips),
            Rule::reg => {
                let s = pair.as_str();
                let indirections = s.bytes().filter(|b| *b == b'r').count() - 1;
                let name = &s[indirections..];
                let reg = mips.get_reg(name)?;
                Ok(Self::Reg { indirections, reg, })
            },
            Rule::alias => {
                let name = pair.as_str().to_owned();
                let reg = mips.get_reg(&name)?;
                Ok(Self::Alias { name, reg })
            },
            _ => panic!("{:?}", pair),
        }
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Reg { indirections, reg } => {
                for _ in 0..*indirections {
                    write!(f, "r")?;
                }
                write!(f, "{}", reg)
            },
            Self::Alias { name, .. } => write!(f, "{}", name),
        }
    }
}

