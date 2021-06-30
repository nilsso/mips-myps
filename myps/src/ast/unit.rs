use ast_traits::{AstNode, AstPairs, AstPair, IntoAst, AstError};

use crate::{MypsParser, MypsError, MypsResult, Pair, Rule};
use crate::ast::{Branch, Stmt};

#[derive(Clone, Debug)]
pub enum Unit {
    Branch(Branch),
    Stmt(Stmt),
    Empty,
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Unit {
    type Output = Self;

    const RULE: Rule = Rule::unit;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::unit => pair.only_inner()?.try_into_ast(),
            Rule::branch => Ok(Self::Branch(pair.try_into_ast()?)),
            Rule::stmt => Ok(Self::Stmt(pair.try_into_ast()?)),
            Rule::empty => Ok(Self::Empty),
            _ => Err(MypsError::pair_wrong_rule("a branch, statement or empty unit", pair)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Line(pub(crate) Unit, pub(crate) Option<String>);

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Line {
    type Output = (usize, Self);

    const RULE: Rule = Rule::line;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::line => {
                let mut pairs = pair.into_inner();
                let spaces = pairs.next_rule(Rule::indent, "an indent")?
                    .as_str().len();
                let unit = pairs.next_rule(Rule::unit, "a unit")?.try_into_ast()?;
                // pairs.next();
                // let unit = Unit::Empty;
                let comment_opt = {
                    let comment_pair = pairs.next_rule(Rule::comment, "a comment")?;
                    let s = comment_pair.as_str();
                    (s.len() > 0).then_some(s.to_owned())
                };
                Ok((spaces, Self(unit, comment_opt)))
            },
            _ => Err(MypsError::pair_wrong_rule("an line", pair)),
        }
    }
}


