use ast_traits::{AstNode, AstPairs, AstPair, IntoAst, AstError};

use crate::{MypsParser, MypsError, MypsResult, Pair, Rule};
use crate::ast::{Lv, Expr, MypsNode};

#[derive(Clone, Debug)]
pub enum Stmt {
    LvRvAsn(Vec<Lv>, Vec<Expr>),
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Stmt {
    type Output = Self;

    const RULE: Rule = Rule::unit;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::stmt => pair.only_inner()?.try_into_ast(),
            Rule::stmt_lvrv_asn => {
                let mut lvs = pair.into_inner().collect::<Vec<_>>();
                if lvs.len() % 2 != 0 {
                    return Err(MypsError::pairs_wrong_parity("an even", lvs.len()));
                }
                let rvs = lvs.drain(lvs.len() / 2..)
                    .map(Expr::try_from_pair)
                    .collect::<MypsResult<Vec<_>>>()?;
                let lvs = lvs.into_iter()
                    .map(Lv::try_from_pair)
                    .collect::<MypsResult<Vec<_>>>()?;
                Ok(Self::LvRvAsn(lvs, rvs))
            },
            _ => Err(MypsError::pair_wrong_rule("a statement", pair)),
        }
    }
}

