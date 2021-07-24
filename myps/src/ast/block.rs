use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};

use mips::MipsResult;

use crate::ast::{Branch, Item};

#[derive(Clone, Debug)]
pub struct Block {
    pub branch: Branch,
    pub items: Vec<Item>,
}

impl Block {
    pub fn new(branch: Branch) -> Self {
        let items = Vec::new();
        Self { branch, items }
    }

    pub fn is_if(&self) -> bool {
        self.branch.is_if()
    }

    pub fn is_elif(&self) -> bool {
        self.branch.is_elif()
    }

    pub fn is_else(&self) -> bool {
        self.branch.is_else()
    }

    pub fn is_if_elif_else(&self) -> bool {
        self.branch.is_if_elif_else()
    }
}

