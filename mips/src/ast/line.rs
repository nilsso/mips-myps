use std::{fmt, fmt::Display};

use ast_traits::{AstNode, AstPairs, IntoAst};

use crate::ast::Stmt;
use crate::{MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub struct Line {
    pub(crate) stmt: Stmt,
    pub(crate) comment_opt: Option<String>,
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Line {
    type Output = Self;

    const RULE: Rule = Rule::line;

    fn try_from_pair(pair: Pair) -> MipsResult<Self> {
        let mut pairs = pair.into_inner();
        let stmt = pairs.next_pair().unwrap().try_into_ast().unwrap();
        let comment_opt = pairs.next().map(|pair| pair.as_str().to_owned());
        Ok(Self { stmt, comment_opt })
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { stmt, comment_opt } = self;

        if let Some(comment) = comment_opt {
            write!(f, "{} {}", stmt, comment)
        } else {
            write!(f, "{}", stmt)
        }
    }
}

