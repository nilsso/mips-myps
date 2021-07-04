use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};
use mips::{MipsResult, Mips};

use crate::ast::{Dev, Expr, IntoMips};
use crate::{MypsError, MypsParser, MypsResult, Pair, Pairs, Rule};
use crate::lexer::MypsLexer;

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
            },
            _ => Err(MypsError::pair_wrong_rule("a variable", pair)),
        }
    }
}

impl<'i> IntoMips<'i> for Var {
    type Output = mips::ast::Reg;

    fn try_into_mips(self, mips: &Mips) -> MipsResult<Self::Output> {
        let Self { key, fixed } = self;
        unimplemented!();
        // mips.try_alias(&key);
        // Ok(Vec::new())
    }
}
