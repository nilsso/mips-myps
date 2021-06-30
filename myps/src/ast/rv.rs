use std::{fmt, fmt::Display};

use ast_traits::{AstError, AstNode, AstPair, IntoAst};
// use mips::ast;

use crate::ast::{Var, MypsNode};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Rv {
    Num(f64),
    Var(Var),
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Rv {
    type Output = Self;

    const RULE: Rule = Rule::rv;

    fn try_from_pair(pair: Pair) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::rv => pair.only_inner()?.try_into_ast(),
            Rule::num => Ok(Self::Num(pair.as_str().parse()?)),
            Rule::var => Ok(Self::Var(pair.try_into_ast()?)),
            _ => Err(MypsError::pair_wrong_rule("an r-value", pair)),
        }
    }
}
