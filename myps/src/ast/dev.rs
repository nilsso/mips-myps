use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};

use crate::ast::{Expr, Var};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Dev {
    Lit(usize),
    Expr(Box<Expr>),
    Var(Var),
    DB,
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Dev {
    type Output = Self;

    const RULE: Rule = Rule::dev_var;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::dev_var | Rule::dev => pair.only_inner().unwrap().try_into_ast(),
            Rule::dev_self => Ok(Self::DB),
            Rule::dev_lit => {
                let index = pair
                    .only_rule(Rule::int, "an index")
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap();
                Ok(Self::Lit(index))
            }
            Rule::dev_expr => {
                let expr = pair.only_inner().unwrap().try_into_ast().unwrap();
                Ok(Self::Expr(Box::new(expr)))
            }
            Rule::var => Ok(Self::Var(pair.try_into_ast().unwrap())),
            _ => return Err(MypsError::pair_wrong_rule("a device", pair)),
        }
    }
}
