use ast_traits::{AstNode, AstPairs, AstPair, IntoAst, AstError};

use crate::{MypsParser, MypsError, MypsResult, Rule, Pair};
use crate::ast::{Block, Stmt};

#[derive(Clone, Debug)]
pub enum Item {
    Block(Block, Option<String>),
    Stmt(Stmt, Option<String>),
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Item {
    type Output = (usize, Item);

    const RULE: Rule = Rule::line;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::single_line => {
                let mut pairs = pair.into_inner();
                let line_pair = pairs.next_pair().unwrap();
                pairs.final_pair().unwrap();
                Self::try_from_pair(line_pair)
            },
            Rule::line => {
                let mut pairs = pair.into_inner();
                let spaces = pairs.next_rule(Rule::indent, "an indent").unwrap()
                    .as_str().len();
                let item_pair = pairs.next_rule(Rule::item, "an item").unwrap().only_inner().unwrap();
                let comment_opt = {
                    let comment_pair = pairs.next_rule(Rule::comment, "a comment").unwrap();
                    let s = comment_pair.as_str();
                    (s.len() > 0).then_some(s.to_owned())
                };
                match item_pair.as_rule() {
                    Rule::branch => {
                        let branch = item_pair.try_into_ast().unwrap();
                        let block = Block::new(branch);
                        Ok((spaces, Self::Block(block, comment_opt)))
                    },
                    Rule::stmt => {
                        let stmt = item_pair.try_into_ast().unwrap();
                        Ok((spaces, Self::Stmt(stmt, comment_opt)))
                    },
                    _ => Err(MypsError::pair_wrong_rule("a branch or unit", item_pair)),
                }
            },
            _ => Err(MypsError::pair_wrong_rule("a branch or unit line", pair)),
        }
    }
}

