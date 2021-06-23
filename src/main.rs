#![feature(stmt_expr_attributes)]
#![feature(map_into_keys_values)]
#![feature(bool_to_option)]
#![feature(box_patterns)]

pub mod ast_traits;
pub mod mips;
pub mod graph;

#[allow(unused_imports)]
use mips::{Mips, ast::{Unit, Arg}, MipsParser, Rule, MipsResult};

fn interpret_source(source: &str, mips: &mut Mips) -> MipsResult<()> {
    for line in source.trim_end().split("\n") {
        mips.intepret_line(line)?;
    }
    Ok(())
}

fn main() {
    #[allow(unused_variables)]
    let header = |h: &str| {
        for _ in 0..100 {
            print!("=");
        }
        println!();
        println!("{}", h);
    };
    #[allow(unused_variables)]
    let print_units = |mips: &Mips| {
        println!("UNITS:");
        for (i, unit) in mips.units.iter().enumerate() {
            println!("{:>3}: {}", i, unit);
            // println!("{:>3}: {:?}", i, unit);
        }
    };
    #[allow(unused_variables)]
    let print_aliases = |mips: &Mips| {
        println!("ALIASES:");
        for (k, a) in mips.aliases.iter() {
            println!("{:>3}: {:?}", k, a);
        }
    };
    #[allow(unused_variables)]
    let print_registers = |mips: &Mips| {
        println!("REGISTERS:");
        for reg in mips.registers.iter() {
            println!("{:?}", reg);
        }
    };

    let path = std::env::args().skip(1).next().unwrap();
    let source = std::fs::read_to_string(path).unwrap();

    let mut mips = Mips::default();

    // println!("{}", source);

    match interpret_source(&source, &mut mips) {
        Ok(()) => {

            print_units(&mips);
            print_registers(&mips);
            print_aliases(&mips);
            // println!("{}", mips.interference_graph());

            // header("OPTIMIZE INSTRUCTIONS");
            // mips.optimize_instructions();
            // print_units(&mips);
            // print_registers(&mips);
            // print_aliases(&mips);
            // println!("{}", mips.interference_graph());

            // header("OPTIMIZING REGISTERS");
            // mips.optimize_registers();
            // print_units(&mips);
            // print_registers(&mips);
            // print_aliases(&mips);
            // println!("{}", mips.interference_graph());
        },
        Err(err) => {
            println!("{}", err);
        }
    }

    // print_units(&mips);
    // let mips2 = mips.deep_clone();
    // header("DEEP CLONE");
    // print_units(&mips);
    // print_registers(&mips2);
    // print_aliases(&mips2);
    // println!("{}", mips.interference_graph());
}
