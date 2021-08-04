#![feature(bool_to_option)]
#![feature(box_patterns)]
#![feature(stmt_expr_attributes)]
#![feature(map_try_insert)]
#![feature(trait_alias)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
use std::{fmt, fmt::Debug};

use pest::Parser;
use pest_derive::Parser;
// use serde::{Deserialize, Serialize};

use ast_traits::{AstNode, AstPairs, AstRule};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct MypsParser;

impl AstRule for Rule {
    fn eoi() -> Self {
        Self::EOI
    }
}

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
pub mod lexer;
