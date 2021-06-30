use ast_traits::{AstNode, AstPairs, AstPair, IntoAst, AstError};

use crate::{MypsParser, MypsError, MypsResult, Pair, Rule};
use crate::ast::Var;

#[derive(Clone, Debug)]
pub enum Lv {
    Var(Var),
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Lv {
    type Output = Lv;

    const RULE: Rule = Rule::lv;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::lv => pair.only_inner()?.try_into_ast(),
            Rule::var => Ok(Self::Var(pair.try_into_ast()?)),
            _ => Err(MypsError::pair_wrong_rule("an l-value", pair)),
        }
    }
}

