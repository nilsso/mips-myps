use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};

use mips::MipsResult;

use crate::ast::{Block, Branch, Stmt};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};
use itertools::chain;

#[derive(Debug)]
pub enum LineItem<'a> {
    Branch(&'a Branch),
    Stmt(&'a Stmt),
}

pub struct LineItemIter<'a> {
    // iter: &'a Item,
    iter: Box<(dyn Iterator<Item = LineItem<'a>> + 'a)>,
}

impl<'a> Iterator for LineItemIter<'a> {
    type Item = LineItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> LineItemIter<'a> {
    pub fn new(item: &'a Item) -> LineItemIter<'a> {
        use std::iter::{empty, once};

        match item {
            Item::Block(Block { branch, items }, _) => {
                if matches!(branch, Branch::Program) {
                    let iter = Box::new(
                        empty().chain(items.iter().flat_map(|item| LineItemIter::new(item))),
                    );
                    Self { iter }
                } else {
                    let iter = Box::new(
                        once(LineItem::Branch(branch))
                            .chain(items.iter().flat_map(|item| LineItemIter::new(item))),
                    );
                    Self { iter }
                }
            }
            Item::Stmt(stmt, _) => {
                let iter = Box::new(once(LineItem::Stmt(stmt)));
                Self { iter }
            }
        }
    }
}

#[derive(Debug)]
pub enum LineItemMut<'a> {
    Branch(&'a mut Branch),
    Stmt(&'a mut Stmt),
}

pub struct LineItemIterMut<'a> {
    // iter: &'a Item,
    iter: Box<(dyn Iterator<Item = LineItemMut<'a>> + 'a)>,
}

impl<'a> Iterator for LineItemIterMut<'a> {
    type Item = LineItemMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> LineItemIterMut<'a> {
    pub fn new(item: &'a mut Item) -> LineItemIterMut<'a> {
        use std::iter::{empty, once};

        match item {
            Item::Block(Block { branch, items }, _) => {
                if matches!(branch, Branch::Program) {
                    let iter = Box::new(
                        empty().chain(items.iter_mut().flat_map(|item| LineItemIterMut::new(item))),
                    );
                    Self { iter }
                } else {
                    let iter = Box::new(
                        once(LineItemMut::Branch(branch))
                            .chain(items.iter_mut().flat_map(|item| LineItemIterMut::new(item))),
                    );
                    Self { iter }
                }
            }
            Item::Stmt(stmt, _) => {
                let iter = Box::new(once(LineItemMut::Stmt(stmt)));
                Self { iter }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Item {
    Block(Block, Option<String>),
    Stmt(Stmt, Option<String>),
}

impl Item {
    pub fn is_if(&self) -> bool {
        match self {
            Self::Block(block, ..) => block.is_if(),
            _ => false,
        }
    }

    pub fn is_elif(&self) -> bool {
        match self {
            Self::Block(block, ..) => block.is_elif(),
            _ => false,
        }
    }

    pub fn is_else(&self) -> bool {
        match self {
            Self::Block(block, ..) => block.is_else(),
            _ => false,
        }
    }

    pub fn is_elif_else(&self) -> bool {
        self.is_elif() || self.is_else()
    }

    pub fn is_if_elif_else(&self) -> bool {
        self.is_if() || self.is_elif() || self.is_else()
    }

    pub fn chain_id(&self) -> Option<usize> {
        match self {
            Self::Block(
                Block {
                    branch: Branch::If { chain_id_opt, .. },
                    ..
                },
                ..,
            ) => chain_id_opt.clone(),
            Self::Block(
                Block {
                    branch: Branch::Elif { chain_id, .. },
                    ..
                },
                ..,
            ) |
            Self::Block(
                Block {
                    branch: Branch::Else { chain_id, .. },
                    ..
                },
                ..,
            ) => Some(*chain_id),
            _ => None,
        }
    }

    pub fn iter(&self) -> LineItemIter {
        LineItemIter::new(self)
    }

    pub fn iter_mut(&mut self) -> LineItemIterMut {
        LineItemIterMut::new(self)
    }
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
            }
            Rule::line => {
                let mut pairs = pair.into_inner();
                let spaces = pairs
                    .next_rule(Rule::indent, "an indent")
                    .unwrap()
                    .as_str()
                    .len();
                let item_pair = pairs
                    .next_rule(Rule::item, "an item")
                    .unwrap()
                    .only_inner()
                    .unwrap();
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
                    }
                    Rule::stmt => {
                        let stmt = item_pair.try_into_ast().unwrap();
                        Ok((spaces, Self::Stmt(stmt, comment_opt)))
                    }
                    _ => Err(MypsError::pair_wrong_rule("a branch or unit", item_pair)),
                }
            }
            _ => Err(MypsError::pair_wrong_rule("a branch or unit line", pair)),
        }
    }
}
