use ast_traits::{AstNode, AstPairs, AstPair, IntoAst, AstError};

use crate::{MypsParser, MypsError, MypsResult, Pair, Rule};
// use crate::ast::Stmt;

#[derive(Clone, Debug)]
pub enum Branch {
    Loop,
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Branch {
    type Output = Self;

    const RULE: Rule = Rule::unit;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::branch => pair.only_inner()?.try_into_ast(),
            Rule::branch_loop => Ok(Self::Loop),
            _ => Err(MypsError::pair_wrong_rule("a branch", pair)),
        }
    }
}

