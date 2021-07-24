use lazy_static::lazy_static;
use regex::{Match, Regex};

use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};

use crate::ast::{Dev, Num, Var};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Lv {
    DevParam { dev: Dev, param: String },
    NetParam { hash: Num, param: String },
    Var(Var),
    Def(String),
}

lazy_static! {
    static ref RESERVED_NAME_PATTERN: Regex = Regex::new(r"(db|((r|d)(0|[1-9]\d*)))").unwrap();
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Lv {
    type Output = Lv;

    const RULE: Rule = Rule::lv;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::lv | Rule::lv_var => pair.only_inner().unwrap().try_into_ast(),
            Rule::lv_dev_param => {
                let mut pairs = pair.into_inner();
                let dev = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let param = pairs.final_pair().unwrap().try_into_ast().unwrap();
                Ok(Self::DevParam { dev, param })
            }
            Rule::lv_net_param => {
                let mut pairs = pair.into_inner();
                let hash = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let param = pairs.final_pair().unwrap().try_into_ast().unwrap();
                Ok(Self::NetParam { hash, param })
            }
            Rule::lv_def => {
                let name = pair.only_inner().unwrap().try_into_ast().unwrap();
                Ok(Self::Def(name))
            }
            Rule::var | Rule::var_fixed => {
                let var = pair.try_into_ast::<Var>().unwrap();
                let invalid_name = RESERVED_NAME_PATTERN
                    .find_at(&var.key, 0)
                    .map(|m| m.end() == var.key.len())
                    .unwrap_or(false);
                if invalid_name {
                    return Err(MypsError::lv_reserved_name(var.key));
                }
                Ok(Self::Var(var))
            }
            _ => Err(MypsError::pair_wrong_rule("an l-value", pair)),
        }
    }
}
