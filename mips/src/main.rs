#![feature(stmt_expr_attributes)]
#![feature(bool_to_option)]
#![feature(box_patterns)]
#![feature(array_methods)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use pest::Parser;

use ast_traits::*;

use mips::ast::*;
use mips::*;

fn main() {
    let path = std::env::args().skip(1).next().unwrap();
    // let source = std::fs::read_to_string(path).unwrap();

    // let path_string = format!("{:?}", path);
    // let f = File::open(path.into()).expect(&path_string);
    // let f = BufReader::new(f);
    // let lines = f.lines().collect::<Result<Vec<_>, _>>().unwrap();

    let mips = Mips::lex_file(path).unwrap();
    // println!("{:#?}", mips.lines);

    // println!("================================================================================");
    let w = (mips.lines.len() as f64 - 1.0).log10().floor().max(0_f64) as usize + 1;
    for (i, line) in mips.lines.iter().enumerate() {
        println!("{:>w$}: {:?}", i, line, w = w);
    }
    println!("--------------------------------------------------------------------------------");
    for (i, line) in mips.lines.iter().enumerate() {
        println!("{:>w$}: {}", i, line, w = w);
    }

    println!("================================================================================");
    let mips = mips
        .optimize(OptimizationConfig {
            optimize_registers: true,
            // optimize_registers: false,
            // remove_comments: true,
            remove_comments: false,
            remove_empty: true,
            // remove_empty: false,
            // remove_empty_comments: true,
            remove_empty_comments: false,

            remove_reg_aliases: true,
            // remove_reg_aliases: false,

            remove_dev_aliases: true,
            // remove_dev_aliases: false,
            remove_defines: true,
            // remove_defines: false,
            remove_tags: true,
            // remove_tags: false,
        })
        .unwrap();
    let w = (mips.lines.len() as f64 - 1.0).log10().floor().max(0_f64) as usize + 1;
    for (i, line) in mips.lines.iter().enumerate() {
        println!("{:>w$}: {:?}", i, line, w = w);
    }
    println!("--------------------------------------------------------------------------------");
    #[allow(unused_variables)]
    for (i, line) in mips.lines.iter().enumerate() {
        // println!("{:>w$}: {}", i, line, w = w);
        println!("{}", line);
    }
}
