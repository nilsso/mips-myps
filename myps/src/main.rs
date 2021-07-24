#![feature(bool_to_option)]
#![allow(unused_imports)]
use pest::Parser;

use ast_traits::*;

use myps::*;
use myps::ast::*;
// use myps::{lexer, MypsParser, MypsError, MypsResult, Pair, Rule};

fn main() {
    // let source = "def x";
    // let rule = Rule::lv;
    // let pairs_res = MypsParser::parse(rule, source);
    // println!("{:#?}", pairs_res);

    let path = std::env::args().skip(1).next().unwrap();
    let source = std::fs::read_to_string(path).unwrap();
    let program_item = lexer::lex_str(&source).unwrap();
    println!("{:#?}", program_item);
    // for line_item in program_item.iter() {
        // println!("{:?}", line_item);
    // }
}
