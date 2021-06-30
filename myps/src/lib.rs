#![feature(stmt_expr_attributes)]
#![feature(bool_to_option)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
use std::{fmt, fmt::Debug};

use pest::Parser;
use pest_derive::Parser;
// use serde::{Deserialize, Serialize};

use ast_traits::{AstNode, AstPairs};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct MypsParser;

pub type Pair<'i> = pest::iterators::Pair<'i, Rule>;
pub type Pairs<'i> = pest::iterators::Pairs<'i, Rule>;

mod error;
pub use error::MypsError;

pub type MypsResult<T> = Result<T, MypsError>;

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for String {
    type Output = String;

    const RULE: Rule = Rule::token;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<String> {
        Ok(pair.as_str().to_owned())
    }
}

pub mod ast;
use ast::{Line, Unit};

#[derive(Clone, Debug)]
pub struct MypsLexer {}

const INDENT_SIZE: usize = 4;

#[derive(Clone, Debug)]
pub enum Item {
    Block(Vec<Item>),
    Line(Line),
}

impl Item {
    pub fn block(branch: Line) -> Self {
        Self::Block(vec![Item::Line(branch)])
    }

    pub fn line(line: Line) -> Self {
        Self::Line(line)
    }

    // pub fn head(&mut self) -> Self {
    // }
}

impl MypsLexer {
    pub fn lex_str(source: &str) -> MypsResult<Self> {

        // let mut items = Item::block();
        let mut block_stack: Vec<Vec<Item>> = vec![vec![]];

        let mut indent_stack = vec![0_usize];
        let mut curr_indent = 0_usize;
        let mut expect_indent = false;

        // fn nest_block(items: &mut Item, indent_stack: &mut Vec<usize>) {
        // }

        for (_i, line_src) in source.trim_end().split("\n").enumerate() {
            let pair = MypsParser::parse(Rule::line, line_src)?.only_rule(Rule::line, "a line")?;
            let (spaces, line) = Line::try_from_pair(pair)?;

            // Handle indent
            if !matches!(line, Line(Unit::Empty, _)) {
                // println!("{:#?}", block_stack);
                let indent = if spaces % INDENT_SIZE != 0 {
                    panic!("Invalid indent size {}", spaces);
                } else {
                    spaces / INDENT_SIZE
                };
                if expect_indent {
                    if indent <= curr_indent {
                        panic!("Expected indent");
                    } else {
                        indent_stack.push(indent);
                        curr_indent = indent;
                        expect_indent = false;
                    }
                } else {
                    while indent < *indent_stack.last().unwrap() {
                        indent_stack.pop();
                        let items = block_stack.pop().unwrap();
                        let head = block_stack.last_mut().unwrap();
                        head.push(Item::Block(items));
                    }
                }
            }

            match &line {
                Line(Unit::Branch(branch), _) => {
                    block_stack.push(vec![Item::line(line)]);
                    expect_indent = true;
                }
                Line(Unit::Stmt(stmt), _) => {
                    let block = block_stack.last_mut().unwrap();
                    block.push(Item::line(line));
                    expect_indent = false;
                }
                Line(Unit::Empty, _) => {
                    let block = block_stack.last_mut().unwrap();
                    block.push(Item::line(line));
                }
            }
        }

        while indent_stack.len() > 1 {
            indent_stack.pop();
            let items = block_stack.pop().unwrap();
            let head = block_stack.last_mut().unwrap();
            head.push(Item::Block(items));
        }
        let program_block = Item::Block(block_stack.pop().unwrap());

        println!("{:#?}", program_block);

        let lexer = Self {};
        Ok(lexer)
    }
}

use std::collections::BTreeMap;

use maplit::btreemap;

use mips::ast::RegBase;

#[derive(Clone, Debug)]
pub struct MypsTranslator {
    next_reg_index: usize,
    aliases: BTreeMap<String, RegBase>,
}

impl Default for MypsTranslator {
    fn default() -> Self {
        let aliases = btreemap! {
            "SP".into() => RegBase::SP,
            "RA".into() => RegBase::RA,
        };
        Self::new(aliases)
    }
}

impl MypsTranslator {
    pub fn new(aliases: BTreeMap<String, RegBase>) -> Self {
        Self {
            next_reg_index: 0,
            aliases,
        }
    }

    pub fn next_reg(&mut self, fixed: bool) -> RegBase {
        let reg_base = RegBase::new_lit(self.next_reg_index, 0, fixed);
        self.next_reg_index += 1;
        reg_base
    }
}
