use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};
use mips::MipsResult;

use crate::ast::{Dev, Expr};
use crate::{MypsError, MypsParser, MypsResult, Pair, Pairs, Rule};

#[derive(Clone, Debug)]
pub struct Var {
    pub key: String,
    pub fixed: bool,
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Var {
    type Output = Self;

    const RULE: Rule = Rule::var;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::var_fixed | Rule::var => {
                let fixed = matches!(pair.as_rule(), Rule::var_fixed);
                let key = pair.only_inner().unwrap().try_into_ast().unwrap();
                Ok(Self { key, fixed })
            }
            _ => Err(MypsError::pair_wrong_rule("a variable", pair)),
        }
    }
}
