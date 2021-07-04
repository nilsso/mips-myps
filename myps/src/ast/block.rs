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
}

