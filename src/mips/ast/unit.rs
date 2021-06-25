use std::{fmt, fmt::Display};

use itertools::join;

use crate::ast_traits::{AstNode, AstPair, AstPairs, IntoAst};
use crate::mips::ast::{Arg, Reg, Num, NumLit, LineAbs, LineRel};
use crate::mips::{MipsError, MipsParser, MipsResult, Pair, Rule, Alias};

macro_rules! def_unit {
    ($(
        ($kind:ident, $n_args:literal, $disp:literal, $try:ident, [$(($ast_kind:ty, $arg_kind:path)),*$(,)*])
    ),*$(,)*) => {
        #[derive(Clone, Debug)]
        pub enum Unit {
            $(
                $kind([Arg; $n_args], Option<String>),
            )*
            Label([Arg; 1], Option<String>),
            Empty([Arg; 0], Option<String>),
        }

        impl Unit {
            $(
                #[allow(unused_variables, unused_mut)]
                pub fn $try(
                    arg_pairs: Vec<Pair>,
                    comment_opt: Option<String>
                ) -> MipsResult<Self> {
                    let n_args = arg_pairs.len();
                    if n_args != $n_args {
                        return Err(MipsError::args_wrong_num($n_args, n_args));
                    }
                    let mut iter = arg_pairs.into_iter();
                    let args: [Arg; $n_args] = [
                        $({
                            let pair = iter.next().unwrap();
                            let ast = <$ast_kind>::try_from_pair(pair)?;
                            let arg = ast.into();
                            // if !matches!(arg, $arg_kind(..)) {
                            //     return Err(MipsError::arg_wrong_kind(stringify!($arg_kind), &arg));
                            // }
                            arg
                        }),*
                    ];
                    let unit = Unit::$kind(args, comment_opt);
                    Ok(unit)
                }
            )*

            pub fn try_from_name(
                instr: &str,
                args: Vec<Pair>,
                comment_opt: Option<String>
            ) -> MipsResult<Self> {
                match instr {
                    $(
                        $disp => Self::$try(args, comment_opt),
                    )*
                    _ => Err(MipsError::instr_unknown(instr)),
                }
            }

            pub fn args(&self) -> &[Arg] {
                match self {
                    $(
                        Self::$kind(args, _) => args,
                    )*
                    Self::Label(args, _) => args,
                    Self::Empty(args, _) => args,
                }
            }

            pub fn args_mut(&mut self) -> &mut [Arg] {
                match self {
                    $(
                        Self::$kind(args, _) => args,
                    )*
                    Self::Label(args, _) => args,
                    Self::Empty(args, _) => args,
                }
            }

            pub fn iter_args(&self) -> impl Iterator<Item = &Arg> {
                self.args().iter()
            }

            pub fn iter_args_mut(&mut self) -> impl Iterator<Item = &mut Arg> {
                self.args_mut().iter_mut()
            }

            // pub fn reduce_args(&mut self) {
            //     for arg in self.iter_args_mut() {
            //         *arg = arg.clone().reduce();
            //     }
            // }

            pub fn comment(&self) -> &Option<String> {
                match self {
                    $(
                        Self::$kind(_, comment_opt) => comment_opt,
                    )*
                    Self::Label(_, comment_opt) => comment_opt,
                    Self::Empty(_, comment_opt) => comment_opt,
                }
            }

            pub fn comment_mut(&mut self) -> &mut Option<String> {
                match self {
                    $(
                        Self::$kind(_, comment_opt) => comment_opt,
                    )*
                    Self::Label(_, comment_opt) => comment_opt,
                    Self::Empty(_, comment_opt) => comment_opt,
                }
            }

            pub fn has_comment(&self) -> bool {
                self.comment().is_some()
            }
        }

        impl Display for Unit {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(
                        Self::$kind(args, comment_opt) => {
                            let args_str = || join(args.iter(), " ");

                            match (args.is_empty(), comment_opt) {
                                (false, Some(comment)) => write!(f, "{} {} {}", $disp, args_str(), comment),
                                (false,          None) => write!(f, "{} {}", $disp, args_str()),
                                ( true, Some(comment)) => write!(f, "{} {}", $disp, comment),
                                ( true,          None) => write!(f, "{}", $disp),
                            }
                        },
                    )*
                    Self::Label([Arg::String(label)], comment_opt) => {
                        if let Some(comment) = comment_opt {
                            write!(f, "{}: {}", label, comment)
                        } else {
                            write!(f, "{}:", label)
                        }
                    }
                    Self::Empty(_, comment_opt) => {
                        if let Some(comment) = comment_opt {
                            write!(f, "{}", comment)
                        } else {
                            Ok(())
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}

#[rustfmt::skip]
def_unit!(
        // Device IO
    // (Bdns,   2, "bdns",   try_bdns,   [(UnitDev, d), (UnitLine, l)]),
    // (Bdnsal, 2, "bdnsal", try_bdnsal, [(UnitDev, d), (UnitLine, l)]),
    // (Bdse,   2, "bdse",   try_bdse,   [(UnitDev, d), (UnitLine, l)]),
    // (Bdseal, 2, "bdseal", try_bdseal, [(UnitDev, d), (UnitLine, l)]),
    // (Brdns,  2, "brdns",  try_brdns,  [(UnitDev, d), (UnitLine, l)]),
    // (Brdse,  2, "brdse",  try_brdse,  [(UnitDev, d), (UnitLine, l)]),
    // (L,      3, "l",      try_l,      [(UnitVar, r), (UnitDev, d), (String, p)]),
    // (Lb,     4, "lb",     try_lb,     [(UnitVar, r), (UnitDevNet, d), (String, p), (Mode, m)]),
    // (Lr
    // (Ls,     4, "ls",     try_ls,     [(UnitVar, r), (UnitDev, d), (UnitNum, i), (String, p)]),
    // (S,      3, "s",      try_s,      [(UnitDev, d), (String, p), (UnitNum, r)]),
    // (Sb,     3, "sb",     try_sb,     [(UnitDevNet, d), (String, p), (UnitNum, r)]),

    // Flow Control, Branches and Jumps
    // (Bap
    // (Bapal
    // (Bapz
    // (Bapzal
    // (Beq
    // (Beqal
    // (Beqz
    // (Beqzal
    // (Bge
    // (Bgeal
    // (Bgez
    // (Bgezal
    // (Bgt
    // (Bgtal
    // (Bgtz
    // (Bgtzal
    // (Ble
    // (Bleal
    // (Blez
    // (Blezal
    // (Blt
    // (Bltal
    // (Bltz
    // (Bltzal
    // (Bna
    // (Bnaal
    // (Bnaz
    // (Bnazal
    // (Bne
    // (Bneal
    // (Bnez
    // (Bnezal

    // (Brap
    // (Brapz
    // (Breq,   3, "breq",   try_breq,   [(UnitNum, a), (UnitNum, b), (UnitLine, l)]),
    // (Breqz,  2, "breqz",  try_breqz,  [(UnitNum, a), (UnitLine, l)]),
    // (Brge,   3, "brge",   try_brge,   [(UnitNum, a), (UnitNum, b), (UnitLine, l)]),
    // (Brgez,  2, "brgez",  try_brgez,  [(UnitNum, a), (UnitLine, l)]),
    // (Brgt,   3, "brgt",   try_brgt,   [(UnitNum, a), (UnitNum, b), (UnitLine, l)]),
    // (Brgtz,  2, "brgtz",  try_brgtz,  [(UnitNum, a), (UnitLine, l)]),
    // (Brle,   3, "brle",   try_brle,   [(UnitNum, a), (UnitNum, b), (UnitLine, l)]),
    // (Brlez,  2, "brlez",  try_brlez,  [(UnitNum, a), (UnitLine, l)]),
    // (Brlt,   3, "brlt",   try_brlt,   [(Num, Arg::Num), (Num, Arg::Num), (LineNum, Arg::Num)]),
    // (Brltz,  2, "brltz",  try_brltz,  [(UnitNum, a), (UnitLine, l)]),
    // (Brna
    // (Brnaz
    // (Brne,   3, "brne",   try_brne,   [(UnitNum, a), (UnitNum, b), (UnitLine, l)]),
    // (Brnez,  2, "brnez",  try_brnez,  [(UnitNum, a), (UnitLine, l)]),
    (J,      1, "j",      try_j,      [(LineAbs, Arg::LineAbs)]),
    // (Jal,    1, "jal",    try_jal,    [(UnitLine, l)]),
    (Jr,     1, "jr",     try_jr,     [(LineRel, Arg::LineRel)]),

    // Variable Selection
    // (Sap
    // (Sapz
    // (Sdns,   2, "sdns",   try_sdns,   [(UnitVar, r), (UnitDev, d)]),
    // (Sdse,   2, "sdse",   try_sdse,   [(UnitVar, r), (UnitDev, d)]),
    // (Select
    // (Seq,    3, "seq",    try_seq,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    // (Seqz,   2, "seqz",   try_seqz,   [(UnitVar, r), (UnitNum, a)]),
    // (Sge,    3, "sge",    try_sge,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    // (Sgez,   2, "sgez",   try_sgez,   [(UnitVar, r), (UnitNum, a)]),
    // (Sgt,    3, "sgt",    try_sgt,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    // (Sgtz,   2, "sgtz",   try_sgtz,   [(UnitVar, r), (UnitNum, a)]),
    // (Sle,    3, "sle",    try_sle,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    // (Slez,   2, "slez",   try_slez,   [(UnitVar, r), (UnitNum, a)]),
    // (Slt,    3, "slt",    try_slt,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    // (Sltz,   2, "sltz",   try_sltz,   [(UnitVar, r), (UnitNum, a)]),
    // (Sna
    // (Snaz
    // (Sne,    3, "sne",    try_sne,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    // (Snez,   2, "snez",   try_snez,   [(UnitVar, r), (UnitNum, a)]),

    // Mathematical Operations
    // (Abs,    2, "abs",    try_abs,    [(UnitVar, r), (UnitNum, a)]),
    // (Acos,   2, "acos",   try_acos,   [(UnitVar, r), (UnitNum, a)]),
    (Add,    3, "add",    try_add,    [(Reg, Arg::Reg), (Num, Arg::Num), (Num, Arg::Num)]),
    // (Asin,   2, "asin",   try_asin,   [(UnitVar, r), (UnitNum, a)]),
    // (Atan,   2, "atan",   try_atan,   [(UnitVar, r), (UnitNum, a)]),
    // (Ceil,   2, "ceil",   try_ceil,   [(UnitVar, r), (UnitNum, a)]),
    // (Cos,    2, "cos",    try_cos,    [(UnitVar, r), (UnitNum, a)]),
    (Div,    3, "div",    try_div,    [(Reg, Arg::Reg), (Num, Arg::Num), (Num, Arg::Num)]),
    // (Exp,    2, "expr",   try_exp,    [(UnitVar, r), (UnitNum, a)]),
    // (Floor,  2, "floor",  try_floor,  [(UnitVar, r), (UnitNum, a)]),
    // (Log,    2, "log",    try_log,    [(UnitVar, r), (UnitNum, a)]),
    // (Max,    3, "max",    try_max,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    // (Min,    3, "min",    try_min,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    // (Mod,    3, "mod",    try_mod,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    (Mul,    3, "mul",    try_mul,    [(Reg, Arg::Reg), (Num, Arg::Num), (Num, Arg::Num)]),
    // (Rand,   1, "rand",   try_rand,   [(UnitVar, r)]),
    // (Round,  2, "round",  try_round,  [(UnitVar, r), (UnitNum, a)]),
    // (Sin,    2, "sin",    try_sin,    [(UnitVar, r), (UnitNum, a)]),
    // (Sqrt,   2, "sqrt",   try_sqrt,   [(UnitVar, r), (UnitNum, a)]),
    (Sub,    3, "sub",    try_sub,    [(Reg, Arg::Reg), (Num, Arg::Num), (Num, Arg::Num)]),
    // (Tan,    2, "tan",    try_tan,    [(UnitVar, r), (UnitNum, a)]),
    // (Trunc,  2, "trunc",  try_trunc,  [(UnitVar, r), (UnitNum, a)]),

    // Logic
    (And,    3, "and",    try_and,    [(Reg, Arg::Var), (Num, Arg::Num), (Num, Arg::Num)]),
    (Nor,    3, "nor",    try_nor,    [(Reg, Arg::Var), (Num, Arg::Num), (Num, Arg::Num)]),
    (Or,     3, "or",     try_or,     [(Reg, Arg::Var), (Num, Arg::Num), (Num, Arg::Num)]),
    (Xor,    3, "xor",    try_xor,    [(Reg, Arg::Var), (Num, Arg::Num), (Num, Arg::Num)]),

    // Stack
    (Peek,   1, "peek",   try_peek,   [(Reg, Arg::Var)]),
    (Pop,    1, "pop",    try_pop,    [(Reg, Arg::Var)]),
    (Push,   1, "push",   try_push,   [(Reg, Arg::Var)]),

    // Misc
    (Alias,  2, "alias",  try_alias,  [(String, Arg::String), (Reg, Arg::Var)]),
    (Define, 2, "define", try_define, [(String, Arg::String), (NumLit, Arg::Num)]),
    (Hcf,    0, "hcf",    try_hcf,    []),
    (Move,   2, "move",   try_move,   [(Reg, Arg::Var), (Num, Arg::Num)]),
    (Sleep,  1, "sleep",  try_sleep,  [(Num, Arg::Num)]),
    (Yield,  0, "yield",  try_yield,  []),
);

/*
 * branches have shared numbers
 * shared numbers are stored in Mips.
 * When a line is removed, need to decrement shared numbers in range
 */

impl Unit {
    // pub fn alias_from(&self) -> Option<(String, Alias)> {
    //     match self {
    //         Unit::Alias([Arg::String(key), Arg::Reg(reg)], _) => {
    //             let reg = Reg::new_alias(key.clone(), reg.reg_shared().clone());
    //             let alias = Alias::Reg(reg);
    //             Some((key.clone(), alias))
    //         }
    //         Unit::Define([Arg::String(key), Arg::Num(num)], _) => {
    //             let alias = Alias::Num(num.clone());
    //             Some((key.clone(), alias))
    //         }
    //         Unit::Label([Arg::String(key), Arg::LineNum(line_num)], _) => {
    //             let alias = Alias::LineNum(line_num.clone());
    //             Some((key.clone(), alias))
    //         },
    //         _ => {
    //             if let Some(Arg::Reg(reg)) = self.iter_args().next() {
    //                 Some((reg.to_string(), Alias::Reg(reg.clone())))
    //             } else {
    //                 None
    //             }
    //         }
    //     }
    // }

    pub fn get_arg(&self, i: usize) -> Option<&Arg> {
        self.args().get(i)
    }

    // pub fn update_regs(&mut self, mips: &Mips) {
    //     for arg in self.iter_args_mut() {
    //         arg.update_reg(mips);
    //     }
    // }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Unit {
    type Output = Self;

    const RULE: Rule = Rule::line;

    fn try_from_pair(pair: Pair) -> MipsResult<Self> {
        let pairs = pair.into_inner();

        let mut unit_pair_opt = None;
        let mut comment_pair_opt = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::unit => unit_pair_opt = Some(pair.only_inner()?),
                Rule::comment => comment_pair_opt = Some(pair.as_str().into()),
                Rule::EOI => {}
                _ => panic!("{:?}", pair),
            }
        }


        if let Some(unit_pair) = unit_pair_opt {
            match unit_pair.as_rule() {
                Rule::label => {
                    let label = unit_pair.as_str().to_string();
                    Ok(Self::Label([label.into()], comment_pair_opt))
                },
                Rule::stmt => {
                    let mut pairs = unit_pair.into_inner();
                    let instr_pair = pairs.next_pair()?;
                    let arg_pairs = pairs.collect();
                    Self::try_from_name(instr_pair.as_str(), arg_pairs, comment_pair_opt)
                },
                _ => unreachable!("{:?}", unit_pair),
            }
        } else {
            Ok(Self::Empty([], comment_pair_opt))
        }
    }
}

// pub struct RegDev;

// impl<'i> MipsNode<'i, Rule, MipsParser> for RegDev {
//     type Output = Arg;

//     const RULE: Rule = Rule::arg;

//     fn try_from_pair(mips: &mut Mips, pair: Pair) -> MipsResult<Self::Output> {
//         match pair.as_rule() {
//             Rule::dev => {
//             },
//             Rule::var => {
//             },
//             Rule::reg => {
//             },
//             _ => panic!(),
//         }
//     }
// }
