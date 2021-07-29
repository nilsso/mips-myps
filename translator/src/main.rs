use translator::Translator;

fn main() {
    use mips::{Mips, OptimizationConfig};

    let myps_path = std::env::args().skip(1).next().unwrap();
    let program_item = myps::lexer::lex_file(&myps_path).unwrap();
    // println!("{:#?}", program_item);

    // println!("{:#?}", program_item);

    // println!("================================================================================");
    let mut translator = Translator::default();
    let lines = translator.translate_item(program_item).unwrap();
    let w = (lines.len() as f64 - 1.0).log10().floor().max(0_f64) as usize + 1;
    // for (i, line) in lines.iter().enumerate() {
    //     println!("{:>w$}: {:?}", i, line, w = w);
    // }
    for (i, line) in lines.iter().enumerate() {
        println!("{:>w$}: {}", i, line, w = w);
    }
    // println!("{:#?}", scopes);

    println!("================================================================================");
    let mips = Mips::default_with_lines(lines).unwrap();
    let w = (mips.lines.len() as f64 - 1.0).log10().floor().max(0_f64) as usize + 1;
    for (i, line) in mips.lines.iter().enumerate() {
        println!("{:>w$}: {:?}", i, line, w = w);
    }
    println!("--------------------------------------------------------------------------------");
    for (i, line) in mips.lines.iter().enumerate() {
        println!("{:>w$}: {}", i, line, w = w);
    }
    println!("{}", mips.interference_graph());
    for (i, (index, (s, e))) in mips.analyze_lifetimes().iter().enumerate() {
        println!("{}: {} ({},{})", i, index, s, e);
    }
    // println!("SCOPES {:?}", mips.scopes);

    println!("================================================================================");
    println!("OPTIMIZE");
    #[rustfmt::skip]
        let mips = mips
        .optimize(OptimizationConfig {
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

            optimize_registers: true,
            // optimize_registers: false,
        },
        )
        .unwrap();

    let w = (mips.lines.len() as f64 - 1.0).log10().floor().max(0_f64) as usize + 1;
    for (i, line) in mips.lines.iter().enumerate() {
        println!("{:>w$}: {:?}", i, line, w = w);
    }
    println!("--------------------------------------------------------------------------------");
    for (_i, line) in mips.lines.iter().enumerate() {
        println!("{:>w$}: {}", _i, line, w = w);
        // println!("{}", line);
    }
    // println!("{}", mips.interference_graph());
    // for (i, (index, (s, e))) in mips.analyze_lifetimes().iter().enumerate() {
    //     println!("{}: {} ({},{})", i, index, s, e);
    // }
    // println!("SCOPES {:?}", mips.scopes);
}
