use std::{fmt, fmt::Display};

use ast_traits::{AstNode, AstPairs, IntoAst};

use crate::ast::Stmt;
use crate::{MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub struct Line {
    pub stmt: Stmt,
    pub comment_opt: Option<String>,
}

impl Line {
    pub fn new(stmt: Stmt, comment_opt: Option<String>) -> Self {
        Self { stmt, comment_opt }
    }

    pub fn new_no_comment(stmt: Stmt) -> Self {
        Self::new(stmt, None)
    }
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
            if matches!(stmt, Stmt::Empty(..)) {
                write!(f, "{}", comment)
            } else {
                write!(f, "{} {}", stmt, comment)
            }
        } else {
            write!(f, "{}", stmt)
        }
    }
}

