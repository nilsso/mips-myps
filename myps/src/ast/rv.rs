use std::{fmt, fmt::Display};

use ast_traits::{AstError, AstNode, AstPair, IntoAst};
use mips::MipsResult;

use crate::ast::{Dev, Expr, Var};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Rv {
    Expr(Expr),
    Dev(Dev),
    Var(Var),
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Rv {
    type Output = Self;

    const RULE: Rule = Rule::rv;

    fn try_from_pair(pair: Pair) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::rv => pair.only_inner().unwrap().try_into_ast(),
            Rule::expr => Ok(Self::Expr(pair.try_into_ast().unwrap())),
            Rule::dev => Ok(Self::Dev(pair.try_into_ast().unwrap())),
            Rule::var => Ok(Self::Var(pair.try_into_ast().unwrap())),
            _ => Err(MypsError::pair_wrong_rule("an r-value (device or expression)", pair)),
        }
    }
}
