#![feature(stmt_expr_attributes)]
#![feature(map_into_keys_values)]
#![feature(bool_to_option)]
#![feature(box_patterns)]
#![feature(array_methods)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

// pub mod ast_traits;
// pub mod mips;

// use mips::*;

#[allow(unused_imports)]
use mips::{Mips, MipsParser, MipsResult, Rule, OptimizationConfig};
// use mips::{Mips, ast::{Unit, Arg}, MipsParser, Rule, MipsResult, OptimizationConfig};

fn main() {
    #[inline]
    fn w(len: usize) -> usize {
        (len as f64 - 1.0).log10().floor().max(0_f64) as usize + 1
    }

    #[allow(unused_variables)]
    let header = |h: &str| {
        for _ in 0..100 {
            print!("=");
        }
        println!();
        println!("{}", h);
    };
    #[allow(unused_variables)]
    let print_lines = |mips: &Mips| {
        println!("LINES:");
        let w = w(mips.lines.len());
        for (i, line) in mips.lines.iter().enumerate() {
            println!("{:>w$}: {}", i, line, w=w);
        }
    };
    #[allow(unused_variables)]
    let print_lines_debug = |mips: &Mips| {
        println!("LINES:");
        let w = w(mips.lines.len());
        for (i, line) in mips.lines.iter().enumerate() {
            println!("{:>w$}: {:?}", i, line, w=w);
        }
    };
    #[allow(unused_variables)]
    let print_lines_both = |mips: &Mips| {
        println!("LINES:");
        let w = w(mips.lines.len());
        for (i, line) in mips.lines.iter().enumerate() {
            println!("{:>w$}: {} {:?}", i, line, line, w=w);
        }
    };
    #[allow(unused_variables)]
    let print_aliases = |mips: &Mips| {
        println!("ALIASES:");
        if let Some(w) = mips.aliases.keys().map(|k| k.to_string().len()).max() {
            for (k, a) in mips.aliases.iter() {
                println!("{:>w$}: {:?}", k, a, w=w);
            }
        }
    };

    let path = std::env::args().skip(1).next().unwrap();
    let source = std::fs::read_to_string(path).unwrap();
    // println!("{}", source);

    fn interpret(source: &str) -> Result<(Mips, Mips), String> {
        let mips = Mips::new(&source)?;
        let mut conf = OptimizationConfig::default();
        // conf.remove_comments = false;
        // conf.remove_empty = false;
        // conf.remove_empty_comments = false;
        // conf.remove_aliases = false;
        // conf.remove_defines = false;
        // conf.remove_tags = false;
        // conf.optimize_registers = false;
        let optimized = mips.optimize(conf)?;
        // let optimized = mips.clone();
        Ok((mips, optimized))
    }

    match interpret(&source) {
        #[allow(unused_variables)]
        Ok((mips, optimized)) => {
            print_lines(&mips);
            // print_lines_debug(&mips);
            // print_lines_both(&mips);
            // print_registers(&mips);
            // print_aliases(&mips);
            // println!("{}", mips.interference_graph());
            // println!("{:#?}", mips.analyze_lifetimes());

            header("OPTIMIZED");
            print_lines(&optimized);
            // print_lines_debug(&optimized);
            // print_lines_both(&mips);
            // print_registers(&optimized);
            // print_aliases(&optimized);
            // println!("{}", optimized.interference_graph());
            // println!("{:#?}", optimized.analyze_lifetimes());

            // for unit in optimized.lines.iter() {
            //     println!("{}", unit);
            // }
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}
