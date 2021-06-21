#![feature(stmt_expr_attributes)]
#![feature(map_into_keys_values)]
#![feature(bool_to_option)]

pub mod ast_traits;
pub mod mips;
pub mod graph;

#[allow(unused_imports)]
use mips::{Mips, ast::{Unit, Arg}, MipsParser, Rule};

fn main() {
    let path = std::env::args().skip(1).next().unwrap();
    let source = std::fs::read_to_string(path).unwrap();

    let mut mips = Mips::default();

    for line in source.trim_end().split("\n") {
        mips.intepret_line(line);
    }

    for (i, unit) in mips.units.iter().enumerate() {
        println!("{:>3}: {}", i, unit);
    }

    for reg in mips.registers.iter() {
        println!("{:?}", reg);
    }

    println!("{}", mips.interference_graph());

    mips.optimize_registers();

    for (i, unit) in mips.units.iter().enumerate() {
        println!("{:>3}: {}", i, unit);
    }
    // for reg in mips.registers.iter() {
    //     println!("{:?}", reg);
    // }
}
