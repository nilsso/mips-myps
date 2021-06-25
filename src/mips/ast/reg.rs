use std::{fmt, fmt::Display};

use crate::ast_traits::{AstNode, AstPair};
// use crate::mips::ast::MipsNode;
use crate::mips::{MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Reg {
    Lit { index: usize, indirections: usize },
    Alias { alias: String },
}

impl Reg {
    pub fn new_lit(index: usize, indirections: usize) -> Self {
        Self::Lit {
            index,
            indirections,
        }
    }

    pub fn new_alias(alias: String) -> Self {
        Self::Alias { alias }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Reg {
    type Output = Reg;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::reg => {
                let s = pair.as_str();
                let indirections = s.bytes().filter(|b| *b == b'r').count() - 1;
                let index = pair.only_inner()?.as_str().parse()?;
                Ok(Self::new_lit(index, indirections))
            }
            Rule::alias => {
                let alias = pair.as_str().to_owned();
                Ok(Self::new_alias(alias))
            }
            _ => Err(MipsError::arg_wrong_kind("a register", pair)),
        }
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Self::Lit {
                indirections,
                index,
            } => {
                for _ in 0..indirections {
                    write!(f, "r")?;
                }
                write!(f, "r{}", index)
            }
            Self::Alias { alias } => write!(f, "{}", alias),
        }
    }
}
