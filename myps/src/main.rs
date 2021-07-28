#![feature(bool_to_option)]
#![allow(unused_imports)]
use std::fs::File;
use std::io::{BufRead, BufReader};

use pest::Parser;

use ast_traits::*;

use myps::*;
use myps::ast::*;
// use myps::{lexer, MypsParser, MypsError, MypsResult, Pair, Rule};

fn main() {
    // let path = std::env::args().skip(1).next().unwrap();
    let path = "./myps/test-scripts/test.myps";
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        println!("{:?}", line);
    }
    // let source = std::fs::read_to_string(path).unwrap();
    // // let source = "fix a = 0";
    // let program_item = lexer::lex_str(&source).unwrap();
    // println!("{:#?}", program_item);
    // // for line_item in program_item.iter() {
    //     // println!("{:?}", line_item);
    // // }
}
