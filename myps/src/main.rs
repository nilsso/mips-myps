#![feature(bool_to_option)]
#![allow(unused_imports)]
use pest::Parser;

use ast_traits::{AstNode, AstPairs, AstPair, IntoAst, AstError};

use myps::{MypsLexer, MypsParser, MypsError, MypsResult, Pair, Rule};

fn main() {
    let path = std::env::args().skip(1).next().unwrap();
    let source = std::fs::read_to_string(path).unwrap();

    // let source = "loop:";
    // println!("{:#?}", MypsParser::parse(Rule::line, source));

    let res = MypsLexer::lex_str(&source);
    // println!("{:#?}", res);

    // println!("{:#?}", Expr::try_from_pair(pair));
}
