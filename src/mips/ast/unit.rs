use std::{fmt, fmt::Display};

use itertools::join;
use pest::iterators::Pair;

use crate::ast_traits::NextPair;
#[allow(unused_imports)]
use crate::mips::ast::{
    MipsNode,
    MipsResult,
    Arg,
    Num,
    Reg,
    Var,
};
use crate::mips::{
    Alias,
    Mips,
    MipsError,
    MipsParser,
    Rule,
};

macro_rules! def_unit {
    (@try_into_arg $mips:ident, $arg_pair:ident, $ast_kind:ty, $arg_kind:path) => {{
        let arg: Arg = <$ast_kind>::try_from_pair($mips, $arg_pair)?.into();
        if !matches!(arg, $arg_kind(..)) {
            panic!("Wrong argument {:?}", arg);
        }
        arg
    }};

    (@test $inner_iter:ident, $arg_kind:path) => {{
        let arg: Arg = $inner_iter.next().unwrap().into();
        if matches!(arg, $arg_kind) {
            panic!();
        }
        arg
    }};

    ($(
        ($kind:ident, $n_args:literal, $disp:literal, $try:ident, [$(($ast_kind:ty, $arg_kind:path)),*$(,)*])
    ),*$(,)*) => {
        #[derive(Clone, Debug)]
        pub enum Unit {
            $(
                $kind([Arg; $n_args], Option<String>),
            )*
            Empty([Arg; 0], Option<String>),
        }

        impl Unit {
            $(
                #[allow(unused_variables, unused_mut)]
                pub fn $try(
                    mips: &mut Mips,
                    arg_pairs: Vec<Pair<Rule>>,
                    comment: Option<String>
                ) -> MipsResult<Self> {
                    let n_args = arg_pairs.len();
                    if n_args != $n_args {
                        return Err(MipsError::args_wrong_amount($n_args, n_args));
                    }
                    let mut iter = arg_pairs.into_iter();
                    let args: [Arg; $n_args] = [
                        $({
                            let pair = iter.next().unwrap();
                            let ast = <$ast_kind>::try_from_pair(mips, pair)?;
                            let arg = ast.into();
                            // if !matches!(arg, $arg_kind(..)) {
                            //     return Err(MipsError::arg_wrong_kind(stringify!($arg_kind), &arg));
                            // }
                            arg
                        }),*
                    ];
                    let unit = Unit::$kind(args, comment);
                    Ok(unit)
                }
            )*

            pub fn try_from_name(
                mips: &mut Mips,
                instr: &str,
                args: Vec<Pair<Rule>>,
                comment: Option<String>
            ) -> MipsResult<Self> {
                match instr {
                    $(
                        $disp => Self::$try(mips, args, comment),
                    )*
                    _ => panic!(),
                }
            }

            pub fn args(&self) -> &[Arg] {
                match self {
                    $(
                        Self::$kind(args, _) => args,
                    )*
                    Self::Empty(args, _) => args,
                }
            }

            pub fn args_mut(&mut self) -> &mut [Arg] {
                match self {
                    $(
                        Self::$kind(args, _) => args,
                    )*
                    Self::Empty(args, _) => args,
                }
            }

            pub fn iter_args(&self) -> impl Iterator<Item = &Arg> {
                self.args().iter()
            }

            pub fn iter_args_mut(&mut self) -> impl Iterator<Item = &mut Arg> {
                self.args_mut().iter_mut()
            }

            pub fn reduce_args(&mut self) {
                for arg in self.iter_args_mut() {
                    *arg = arg.clone().reduce();
                }
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
                    Self::Empty(_, comment_opt) => {
                        if let Some(comment) = comment_opt {
                            write!(f, "{}", comment)
                        } else {
                            Ok(())
                        }
                    }
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
    (Brlt,   3, "brlt",   try_brlt,   [(Num, Arg::Num), (Num, Arg::Num), (Num, Arg::Num)]),
    // (Brltz,  2, "brltz",  try_brltz,  [(UnitNum, a), (UnitLine, l)]),
    // (Brna
    // (Brnaz
    // (Brne,   3, "brne",   try_brne,   [(UnitNum, a), (UnitNum, b), (UnitLine, l)]),
    // (Brnez,  2, "brnez",  try_brnez,  [(UnitNum, a), (UnitLine, l)]),
    // (J,      1, "j",      try_j,      [(UnitLine, l)]),
    // (Jal,    1, "jal",    try_jal,    [(UnitLine, l)]),
    // (Jr,     1, "jr",     try_jr,     [(UnitLine, l)]),

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
    (Add,    3, "add",    try_add,    [(Reg, Arg::Var), (Num, Arg::Num), (Num, Arg::Num)]),
    // (Asin,   2, "asin",   try_asin,   [(UnitVar, r), (UnitNum, a)]),
    // (Atan,   2, "atan",   try_atan,   [(UnitVar, r), (UnitNum, a)]),
    // (Ceil,   2, "ceil",   try_ceil,   [(UnitVar, r), (UnitNum, a)]),
    // (Cos,    2, "cos",    try_cos,    [(UnitVar, r), (UnitNum, a)]),
    (Div,    3, "div",    try_div,    [(Num, Arg::Num), (Num, Arg::Num), (Num, Arg::Num)]),
    // (Exp,    2, "expr",   try_exp,    [(UnitVar, r), (UnitNum, a)]),
    // (Floor,  2, "floor",  try_floor,  [(UnitVar, r), (UnitNum, a)]),
    // (Log,    2, "log",    try_log,    [(UnitVar, r), (UnitNum, a)]),
    // (Max,    3, "max",    try_max,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    // (Min,    3, "min",    try_min,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    // (Mod,    3, "mod",    try_mod,    [(UnitVar, r), (UnitNum, a), (UnitNum, b)]),
    (Mul,    3, "mul",    try_mul,    [(Num, Arg::Num), (Num, Arg::Num), (Num, Arg::Num)]),
    // (Rand,   1, "rand",   try_rand,   [(UnitVar, r)]),
    // (Round,  2, "round",  try_round,  [(UnitVar, r), (UnitNum, a)]),
    // (Sin,    2, "sin",    try_sin,    [(UnitVar, r), (UnitNum, a)]),
    // (Sqrt,   2, "sqrt",   try_sqrt,   [(UnitVar, r), (UnitNum, a)]),
    (Sub,    3, "sub",    try_sub,    [(Num, Arg::Num), (Num, Arg::Num), (Num, Arg::Num)]),
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
    (Define, 2, "define", try_define, [(String, Arg::String), (Num, Arg::Num)]),
    (Hcf,    0, "hcf",    try_hcf,    []),
    (Move,   2, "move",   try_move,   [(Reg, Arg::Var), (Num, Arg::Num)]),
    (Sleep,  1, "sleep",  try_sleep,  [(Num, Arg::Num)]),
    (Yield,  0, "yield",  try_yield,  []),
);

impl Unit {
    pub fn alias_from(&self) -> Option<(String, Alias)> {
        match self {
            Unit::Alias([Arg::String(a), Arg::Var(v)], _) => {
                let name = a.clone();
                let var = Var::Alias {
                    name: name.clone(),
                    reg: v.reg().clone(),
                };
                let alias = Alias::Var(var);
                Some((name, alias))
            }
            Unit::Define([Arg::String(a), Arg::Num(n)], _) => {
                let name = a.clone();
                let alias = Alias::Num(n.clone());
                Some((name, alias))
            }
            _ => {
                if let Some(Arg::Var(var)) = self.iter_args().next() {
                    Some((var.to_string(), Alias::Var(var.clone())))
                } else {
                    None
                }
            }
        }
    }
}

impl<'i> MipsNode<'i, Rule, MipsParser> for Unit {
    type Output = Self;

    const RULE: Rule = Rule::line;

    fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> MipsResult<Self::Output> {
        let pairs = pair.into_inner();

        let mut stmt_pair_opt = None;
        let mut comment_pair_opt = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::stmt => stmt_pair_opt = Some(pair),
                Rule::comment => comment_pair_opt = Some(pair.as_str().into()),
                Rule::EOI => {}
                _ => panic!("{:?}", pair),
            }
        }

        let unit = if let Some(stmt_pair) = stmt_pair_opt {
            let mut pairs = stmt_pair.into_inner();

            let instr_pair = pairs.next_pair()?;
            let arg_pairs = pairs.collect();

            Self::try_from_name(mips, instr_pair.as_str(), arg_pairs, comment_pair_opt)?
        } else {
            Self::Empty([], comment_pair_opt)
        };

        // match &unit {
        //     Unit::Alias([Arg::String(k), Arg::Var(var)], _) => {
        //         mips.aliases.insert(k.clone(), Alias::Reg(reg.clone()));
        //     }
        //     _ => {}
        // }

        Ok(unit)
    }
}

// pub struct RegDev;

// impl<'i> MipsNode<'i, Rule, MipsParser> for RegDev {
//     type Output = Arg;

//     const RULE: Rule = Rule::arg;

//     fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> MipsResult<Self::Output> {
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
