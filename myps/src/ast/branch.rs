use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};

use crate::ast::{Expr, Var};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Branch {
    Program,
    Loop,
    While(Expr),
    If(Expr),
    Elif(Expr),
    Else,
    For(Var, Expr, Expr, Expr),
    Tag(String),
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Branch {
    type Output = Self;

    const RULE: Rule = Rule::branch;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::branch => pair.only_inner().unwrap().try_into_ast(),
            Rule::branch_loop => Ok(Self::Loop),
            Rule::branch_while => {
                let cond = pair.only_inner().unwrap().try_into_ast().unwrap();
                Ok(Self::While(cond))
            }
            Rule::branch_if => {
                let cond = pair.only_inner().unwrap().try_into_ast().unwrap();
                Ok(Self::If(cond))
            }
            Rule::branch_elif => {
                let cond = pair.only_inner().unwrap().try_into_ast().unwrap();
                Ok(Self::Elif(cond))
            }
            Rule::branch_else => Ok(Self::Else),
            Rule::branch_for => {
                let mut pairs = pair.into_inner();
                let i = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let s = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let e = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let step = pairs
                    .next()
                    .map(Expr::try_from_pair)
                    .transpose()
                    .unwrap()
                    .unwrap_or(1.into());
                Ok(Self::For(i, s, e, step))
            }
            Rule::branch_tag => {
                let tag = pair.only_inner().unwrap().try_into_ast().unwrap();
                Ok(Self::Tag(tag))
            }
            _ => Err(MypsError::pair_wrong_rule("a branch", pair)),
        }
    }
}
