#![feature(stmt_expr_attributes)]
#![feature(map_into_keys_values)]
#![feature(bool_to_option)]
#![feature(box_patterns)]
#![feature(array_methods)]
#![allow(unused_imports)]

pub mod ast_traits;
pub mod graph;
pub mod mips;

#[allow(unused_imports)]
use mips::{Mips, MipsParser, MipsResult, Rule, OptimizationConfig};
// use mips::{Mips, ast::{Unit, Arg}, MipsParser, Rule, MipsResult, OptimizationConfig};

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
        }
    };
    #[allow(unused_variables)]
    let print_units_debug = |mips: &Mips| {
        println!("UNITS:");
        for (i, unit) in mips.units.iter().enumerate() {
            println!("{:>3}: {:?}", i, unit);
        }
    };
    // #[allow(unused_variables)]
    // let print_aliases = |mips: &Mips| {
    //     println!("ALIASES:");
    //     for (k, a) in mips.aliases.iter() {
    //         println!("{:>3}: {:?}", k, a);
    //     }
    // };

    let path = std::env::args().skip(1).next().unwrap();
    let source = std::fs::read_to_string(path).unwrap();
    // println!("{}", source);

    match Mips::new(&source) {
        Ok(mut mips) => {
            print_units(&mips);
            // print_units_debug(&mips);
            // print_registers(&mips);
            // print_aliases(&mips);
            // println!("{}", mips.interference_graph());

            header("OPTIMIZE");
            #[allow(unused_variables)]
            let mut conf = OptimizationConfig::default();
            // conf.remove_comments = false;
            // conf.remove_empty = false;
            conf.reduce_aliases = false;
            conf.reduce_defines = false;
            conf.reduce_labels = false;
            conf.optimize_registers = false;

            mips.optimize(conf);

            print_units(&mips);
            print_units_debug(&mips);
            // print_registers(&mips);
            // print_aliases(&mips);
            // println!("{}", mips.interference_graph());
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}
