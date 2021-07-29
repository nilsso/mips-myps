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
    let source= "123.all.Temperature.sum";
    println!("{:?}", myps::MypsParser::parse(Rule::num_net_param, source));

    // let path = "./myps/test-scripts/test.myps";
    // let file = File::open(path).unwrap();
    // let reader = BufReader::new(file);
    //
    // for line in reader.lines() {
    //     println!("{:?}", line);
    // }
}
