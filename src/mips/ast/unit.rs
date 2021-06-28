use std::{fmt, fmt::Display};

use itertools::join;

use crate::ast_traits::{AstNode, AstPair, AstPairs, IntoAst};
use crate::mips::ast::{
    Arg, BatchMode, Dev, DevOrReg, LineAbs, LineRel, MipsNode, Num, NumLit, ReagentMode, Reg,
    RegBase, RegLit,
};
use crate::mips::{Mips, MipsError, MipsParser, MipsResult, Pair, Rule};

macro_rules! def_unit {
    ($(
        ($kind:ident, $n_args:literal, $disp:literal, $try:ident, [$($ast_kind:ty),*$(,)*])
    ),*$(,)*) => {
        #[derive(Clone, Debug)]
        pub enum Unit {
            $(
                $kind([Arg; $n_args], Option<String>),
            )*
            Tag([Arg; 1], Option<String>),
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
                            ast.into()
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
                    Self::Tag(args, _) => args,
                    Self::Empty(args, _) => args,
                }
            }

            pub fn args_mut(&mut self) -> &mut [Arg] {
                match self {
                    $(
                        Self::$kind(args, _) => args,
                    )*
                    Self::Tag(args, _) => args,
                    Self::Empty(args, _) => args,
                }
            }

            pub fn comment(&self) -> &Option<String> {
                match self {
                    $(
                        Self::$kind(_, comment_opt) => comment_opt,
                    )*
                    Self::Tag(_, comment_opt) => comment_opt,
                    Self::Empty(_, comment_opt) => comment_opt,
                }
            }

            pub fn comment_mut(&mut self) -> &mut Option<String> {
                match self {
                    $(
                        Self::$kind(_, comment_opt) => comment_opt,
                    )*
                    Self::Tag(_, comment_opt) => comment_opt,
                    Self::Empty(_, comment_opt) => comment_opt,
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
                    Self::Tag([Arg::String(tag)], comment_opt) => {
                        if let Some(comment) = comment_opt {
                            write!(f, "{}: {}", tag, comment)
                        } else {
                            write!(f, "{}:", tag)
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
    (Bdns,   2, "bdns",   try_bdns,   [Dev, LineAbs]),
    (Bdnsal, 2, "bdnsal", try_bdnsal, [Dev, LineAbs]),
    (Bdse,   2, "bdse",   try_bdse,   [Dev, LineAbs]),
    (Bdseal, 2, "bdseal", try_bdseal, [Dev, LineAbs]),
    (Brdns,  2, "brdns",  try_brdns,  [Dev, LineRel]),
    (Brdse,  2, "brdse",  try_brdse,  [Dev, LineRel]),
    (L,      3, "l",      try_l,      [Reg, Dev, String]),
    (Lb,     4, "lb",     try_lb,     [Reg, Num, String, BatchMode]),
    (Lr,     4, "lr",     try_lr,     [Reg, Dev, ReagentMode, String]),
    (Ls,     4, "ls",     try_ls,     [Reg, Dev, Num, String]),
    (S,      3, "s",      try_s,      [Dev, String, Num]),
    (Sb,     3, "sb",     try_sb,     [Num, Dev, Num]),

    // Flow Control, Branches and Jumps
    (Bap,    4, "bap",    try_bap,    [Num, Num, Num, LineAbs]),
    (Bapal,  4, "bapal",  try_bapal,  [Num, Num, Num, LineAbs]),
    (Bapz,   3, "bapz",   try_bapz,   [Num, Num, LineAbs]),
    (Bapzal, 3, "bapzal", try_bapzal, [Num, Num, LineAbs]),
    (Beq,    3, "beq",    try_beq,    [Num, Num, LineAbs]),
    (Beqal,  3, "beqal",  try_beqal,  [Num, Num, LineAbs]),
    (Beqz,   2, "beqz",   try_beqz,   [Num, LineAbs]),
    (Beqzal, 2, "beqzal", try_beqzal, [Num, LineAbs]),
    (Bge,    3, "bge",    try_bge,    [Num, Num, LineAbs]),
    (Bgeal,  3, "bgeal",  try_bgeal,  [Num, Num, LineAbs]),
    (Bgez,   2, "bgez",   try_bgez,   [Num, LineAbs]),
    (Bgezal, 2, "bgezal", try_bgezal, [Num, LineAbs]),
    (Bgt,    3, "bgt",    try_bgt,    [Num, Num, LineAbs]),
    (Bgtal,  3, "bgtal",  try_bgtal,  [Num, Num, LineAbs]),
    (Bgtz,   2, "bgtz",   try_bgtz,   [Num, LineAbs]),
    (Bgtzal, 2, "bgtzal", try_bgtzal, [Num, LineAbs]),
    (Ble,    3, "ble",    try_ble,    [Num, Num, LineAbs]),
    (Bleal,  3, "bleal",  try_bleal,  [Num, Num, LineAbs]),
    (Blez,   2, "blez",   try_blez,   [Num, LineAbs]),
    (Blezal, 2, "blezal", try_blezal, [Num, LineAbs]),
    (Blt,    3, "blt",    try_blt,    [Num, Num, LineAbs]),
    (Bltal,  3, "bltal",  try_bltal,  [Num, Num, LineAbs]),
    (Bltz,   2, "bltz",   try_bltz,   [Num, LineAbs]),
    (Bltzal, 2, "bltzal", try_bltzal, [Num, LineAbs]),
    (Bna,    4, "bna",    try_bna,    [Num, Num, Num, LineAbs]),
    (Bnaal,  4, "bnaal",  try_bnaal,  [Num, Num, Num, LineAbs]),
    (Bnaz,   3, "bnaz",   try_bnaz,   [Num, Num, LineAbs]),
    (Bnazal, 3, "bnazal", try_bnazal, [Num, Num, LineAbs]),
    (Bne,    3, "bne",    try_bne,    [Num, Num, LineAbs]),
    (Bneal,  3, "bneal",  try_bneal,  [Num, Num, LineAbs]),
    (Bnez,   2, "bnez",   try_bnez,   [Num, LineAbs]),
    (Bnezal, 2, "bnezal", try_bnezal, [Num, LineAbs]),

    (Brap,   4, "brap",   try_brap,   [Num, Num, Num, LineRel]),
    (Brapz,  3, "brapz",  try_brapz,  [Num, Num, LineRel]),
    (Breq,   3, "breq",   try_breq,   [Num, Num, LineRel]),
    (Breqz,  2, "breqz",  try_breqz,  [Num, LineRel]),
    (Brge,   3, "brge",   try_brge,   [Num, Num, LineRel]),
    (Brgez,  2, "brgez",  try_brgez,  [Num, LineRel]),
    (Brgt,   3, "brgt",   try_brgt,   [Num, Num, LineRel]),
    (Brgtz,  2, "brgtz",  try_brgtz,  [Num, LineRel]),
    (Brle,   3, "brle",   try_brle,   [Num, Num, LineRel]),
    (Brlez,  2, "brlez",  try_brlez,  [Num, LineRel]),
    (Brlt,   3, "brlt",   try_brlt,   [Num, Num, LineRel]),
    (Brltz,  2, "brltz",  try_brltz,  [Num, LineRel]),
    (Brna,   4, "brna",   try_brna,   [Num, Num, Num, LineRel]),
    (Brnaz,  3, "brnaz",  try_brnaz,  [Num, Num, LineRel]),
    (Brne,   3, "brne",   try_brne,   [Num, Num, LineRel]),
    (Brnez,  2, "brnez",  try_brnez,  [Num, LineRel]),
    (J,      1, "j",      try_j,      [LineAbs]),
    (Jal,    1, "jal",    try_jal,    [LineAbs]),
    (Jr,     1, "jr",     try_jr,     [LineRel]),

    // Variable Selection
    (Sap,    4, "sap",    try_sap,    [Reg, Num, Num, Num]),
    (Sapz,   3, "sapz",   try_sapz,   [Reg, Num, Num]),
    (Sdns,   2, "sdns",   try_sdns,   [Reg, Dev]),
    (Sdse,   2, "sdse",   try_sdse,   [Reg, Dev]),
    (Select, 4, "select", try_select, [Reg, Num, Num, Num]),
    (Seq,    3, "seq",    try_seq,    [Reg, Num, Num]),
    (Seqz,   2, "seqz",   try_seqz,   [Reg, Num]),
    (Sge,    3, "sge",    try_sge,    [Reg, Num, Num]),
    (Sgez,   2, "sgez",   try_sgez,   [Reg, Num]),
    (Sgt,    3, "sgt",    try_sgt,    [Reg, Num, Num]),
    (Sgtz,   2, "sgtz",   try_sgtz,   [Reg, Num]),
    (Sle,    3, "sle",    try_sle,    [Reg, Num, Num]),
    (Slez,   2, "slez",   try_slez,   [Reg, Num]),
    (Slt,    3, "slt",    try_slt,    [Reg, Num, Num]),
    (Sltz,   2, "sltz",   try_sltz,   [Reg, Num]),
    (Sna,    4, "sna",    try_sna,    [Reg, Num, Num, Num]),
    (Snaz,   3, "snaz",   try_snaz,   [Reg, Num, Num]),
    (Sne,    3, "sne",    try_sne,    [Reg, Num, Num]),
    (Snez,   2, "snez",   try_snez,   [Reg, Num]),

    // Mathematical Operations
    (Abs,    2, "abs",    try_abs,    [Reg, Num]),
    (Acos,   2, "acos",   try_acos,   [Reg, Num]),
    (Add,    3, "add",    try_add,    [Reg, Num, Num]),
    (Asin,   2, "asin",   try_asin,   [Reg, Num]),
    (Atan,   2, "atan",   try_atan,   [Reg, Num]),
    (Ceil,   2, "ceil",   try_ceil,   [Reg, Num]),
    (Cos,    2, "cos",    try_cos,    [Reg, Num]),
    (Div,    3, "div",    try_div,    [Reg, Num, Num]),
    (Exp,    2, "expr",   try_exp,    [Reg, Num]),
    (Floor,  2, "floor",  try_floor,  [Reg, Num]),
    (Log,    2, "log",    try_log,    [Reg, Num]),
    (Max,    3, "max",    try_max,    [Reg, Num, Num]),
    (Min,    3, "min",    try_min,    [Reg, Num, Num]),
    (Mod,    3, "mod",    try_mod,    [Reg, Num, Num]),
    (Mul,    3, "mul",    try_mul,    [Reg, Num, Num]),
    (Rand,   1, "rand",   try_rand,   [Reg]),
    (Round,  2, "round",  try_round,  [Reg, Num]),
    (Sin,    2, "sin",    try_sin,    [Reg, Num]),
    (Sqrt,   2, "sqrt",   try_sqrt,   [Reg, Num]),
    (Sub,    3, "sub",    try_sub,    [Reg, Num, Num]),
    (Tan,    2, "tan",    try_tan,    [Reg, Num]),
    (Trunc,  2, "trunc",  try_trunc,  [Reg, Num]),

    // Logic
    (And,    3, "and",    try_and,    [Reg, Num, Num]),
    (Nor,    3, "nor",    try_nor,    [Reg, Num, Num]),
    (Or,     3, "or",     try_or,     [Reg, Num, Num]),
    (Xor,    3, "xor",    try_xor,    [Reg, Num, Num]),

    // Stack
    (Peek,   1, "peek",   try_peek,   [Reg]),
    (Pop,    1, "pop",    try_pop,    [Reg]),
    (Push,   1, "push",   try_push,   [Reg]),

    // Misc
    (Alias,  2, "alias",  try_alias,  [String, DevOrReg]),
    (Define, 2, "define", try_define, [String, NumLit]),
    (Hcf,    0, "hcf",    try_hcf,    []),
    (Move,   2, "move",   try_move,   [Reg, Num]),
    (Sleep,  1, "sleep",  try_sleep,  [Num]),
    (Yield,  0, "yield",  try_yield,  []),
);

/*
 * branches have shared numbers
 * shared numbers are stored in Mips.
 * When a line is removed, need to decrement shared numbers in range
 */

impl Unit {
    pub fn get_arg(&self, i: usize) -> Option<&Arg> {
        self.args().get(i)
    }

    pub fn iter_args(&self) -> impl Iterator<Item = &Arg> {
        self.args().iter()
    }

    pub fn iter_args_mut(&mut self) -> impl Iterator<Item = &mut Arg> {
        self.args_mut().iter_mut()
    }

    pub fn has_comment(&self) -> bool {
        self.comment().is_some()
    }

    pub fn reduce_args(&mut self, mips: &Mips) -> MipsResult<()> {
        for arg in self.iter_args_mut() {
            if let Some(key) = arg.as_alias() {
                if !mips.present_aliases.contains(key) {
                    *arg = arg.clone().reduce(mips)?;
                }
            }
        }
        Ok(())
    }
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
                Rule::tag => {
                    let tag = unit_pair.as_str().to_string();
                    Ok(Self::Tag([tag.into()], comment_pair_opt))
                }
                Rule::stmt => {
                    let mut pairs = unit_pair.into_inner();
                    let instr_pair = pairs.next_pair()?;
                    let arg_pairs = pairs.collect();
                    let mut unit =
                        Self::try_from_name(instr_pair.as_str(), arg_pairs, comment_pair_opt)?;
                    if let Unit::Alias(
                        [_, Arg::Reg(Reg::Base(RegBase::Lit(RegLit { fixed, .. })))],
                        Some(comment),
                    ) = &mut unit
                    {
                        if comment.find("FIX").is_some() {
                            *fixed = true;
                        }
                    };
                    Ok(unit)
                }
                _ => unreachable!("{:?}", unit_pair),
            }
        } else {
            Ok(Self::Empty([], comment_pair_opt))
        }
    }
}
