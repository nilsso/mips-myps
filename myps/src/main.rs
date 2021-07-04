#![feature(bool_to_option)]
#![allow(unused_imports)]
use pest::Parser;

use ast_traits::{AstNode, AstPairs, AstPair, IntoAst, AstError};

use myps::{MypsParser, Rule, lexer};
use myps::ast::*;
// use myps::{lexer, MypsParser, MypsError, MypsResult, Pair, Rule};

fn main() {
    // let source = "d0.Setting";
    // let rule = Rule::rv;
    // // type Ast = Num;
    // // let rule = Rule::item;
    // type Ast = Item;
    // let pairs_res = MypsParser::parse(rule, source);
    // println!("{:#?}", pairs_res);
    // let pairs = pairs_res.unwrap();
    // let pair = pairs.only_pair().unwrap();
    // let line_pair = pair.first_inner().unwrap();
    // println!("{:#?}", Ast::try_from_pair(line_pair));

    let path = std::env::args().skip(1).next().unwrap();
    let source = std::fs::read_to_string(path).unwrap();
    let program_res = lexer::lex_str(&source);
    println!("{:#?}", program_res);
}
