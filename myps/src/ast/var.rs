use ast_traits::{AstError, AstNode, AstPair, IntoAst};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub struct Var(String, bool);

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Var {
    type Output = Self;

    const RULE: Rule = Rule::rv;

    fn try_from_pair(pair: Pair) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::var => {
                let var_pair = pair.only_inner()?;
                let rule = var_pair.as_rule();
                let token = var_pair.only_inner()?.as_str().to_owned();
                #[rustfmt::skip]
                match rule {
                    Rule::var_fixed   => Ok(Self(token, true)),
                    Rule::var_unfixed => Ok(Self(token, false)),
                    _ => unreachable!(),
                }
            }
            _ => Err(MypsError::pair_wrong_rule("a fixed/unfixed var", pair)),
        }
    }
}
