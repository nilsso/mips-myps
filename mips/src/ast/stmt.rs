use std::{fmt, fmt::Display};

use itertools::join;

use ast_traits::{AstNode, AstPairs};

use crate::ast::{Arg, MipsNode};
use crate::{Aliases, MipsError, MipsParser, MipsResult, Pair, Rule};

macro_rules! def_stmt {
    ($(
        ($name:ident, $disp:literal, $n_args:literal, $expected:literal, [$($arg_kind:ty),*$(,)*])
    ),*$(,)*) => {
        #[derive(Clone, Debug)]
        pub enum Stmt {
            $(
                $name([Arg; $n_args]),
            )*
            Tag([Arg; 1]),
            Empty([Arg; 0]),
        }

        impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Stmt {
            type Output = Self;

            const RULE: Rule = Rule::item;

            fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
                match pair.as_rule() {
                    Rule::empty => {
                        Ok(Self::Empty([]))
                    },
                    Rule::tag => {
                        let tag = pair.as_str().to_owned();
                        Ok(Self::Tag([tag.into()]))
                    }
                    Rule::stmt => {
                        let mut pairs = pair.into_inner();
                        let name_str = pairs.next_pair().unwrap().as_str();
                        let arg_pairs = pairs.collect::<Vec<_>>();
                        #[allow(dead_code, unused_variables, unused_mut)]
                        match name_str {
                            $(
                                $disp => {
                                    if $n_args != arg_pairs.len() {
                                        return Err(MipsError::args_wrong_num(
                                            name_str,
                                            $n_args,
                                            arg_pairs.len()
                                        ));
                                    }
                                    let found = join(arg_pairs.iter().map(|pair| {
                                            format!("{:?}", pair.as_rule())
                                        }), ",");
                                    let mut pairs = arg_pairs.into_iter();
                                    let args: [Arg; $n_args] = [
                                        $({
                                            let pair = pairs.next().unwrap();
                                            let ast = <$arg_kind>::try_from_pair(pair)
                                                .map_err(|_| {
                                                    MipsError::args_wrong_kinds(
                                                        name_str,
                                                        $expected,
                                                        &found,
                                                    )
                                                })
                                                .unwrap();
                                            ast.into()
                                        }),*
                                    ];
                                    let stmt = Self::$name(args);
                                    Ok(stmt)
                                }
                            )*
                            _ => unreachable!(),
                        }
                    }
                    _ => Err(MipsError::pair_wrong_rule("a MIPS instruction", pair)),
                }
            }
        }

        impl<'i> Stmt {
            // $(
            //     #[allow(unused_variables, unused_mut)]
            //     pub fn $try(
            //         arg_pairs: Vec<Pair<'i>>,
            //     ) -> MipsResult<Self> {
            //         let n_args = arg_pairs.len();
            //         if n_args != $n_args {
            //             return Err(MipsError::args_wrong_num($disp, $n_args, n_args));
            //         }
            //         // let mut iter = arg_pairs.into_iter();
            //         // let args: [Arg; $n_args] = [
            //         //     $({
            //         //         let pair = iter.next().unwrap();
            //         //         let ast = <$ast_kind as AstNode<'i, Rule, MipsParser, MipsError>>::try_from_pair(pair)?;
            //         //         ast.into()
            //         //     }),*
            //         // ];
            //         // let stmt = Stmt::$kind(args);
            //         // Ok(stmt)
            //         unimplemented!();
            //     }
            // )*

            // pub fn try_from_name(
            //     instr: &str,
            //     args: Vec<Pair>,
            // ) -> MipsResult<Self> {
            //     match instr {
            //         $(
            //             $disp => Self::$try(args),
            //         )*
            //         _ => Err(MipsError::instr_unknown(instr)),
            //     }
            // }

            // pub fn try_from_name_args(
            //     instr: &str,
            //     args: Vec<Arg>,
            // ) -> MipsResult<Self> {
            //     match instr {
            //         $(
            //             $disp => Self::$try(args),
            //         )*
            //         _ => Err(MipsError::instr_unknown(instr)),
            //     }
            // }

            pub fn args(&self) -> &[Arg] {
                match self {
                    $(
                        Self::$name(args) => args,
                    )*
                    Self::Tag(args) => args,
                    Self::Empty(args) => args,
                }
            }

            pub fn args_mut(&mut self) -> &mut [Arg] {
                match self {
                    $(
                        Self::$name(args) => args,
                    )*
                    Self::Tag(args) => args,
                    Self::Empty(args) => args,
                }
            }
        }

        impl Display for Stmt {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(
                        Self::$name(args) => {
                            let args_str = || join(args.iter(), " ");
                            if args.is_empty() {
                                write!(f, "{}", $disp)
                            } else {
                                write!(f, "{} {}", $disp, args_str())
                            }
                        },
                    )*
                    Self::Tag(args) => {
                        write!(f, "{}:", args.first().unwrap())
                    }
                    Self::Empty(_) => Ok(()),
                }
            }
        }
    }
}

#[rustfmt::skip]
use crate::ast::{
    DevOrReg as DoR,
    Reg as R,
    Dev as D,
    Num as N,
    LineAbs as LA,
    LineRel as LR,
};
use String as S;

#[rustfmt::skip]
def_stmt!(
    // Device IO
    (Bdns,   "bdns",   2, "dev,num",           [D, LA]),
    (Bdnsal, "bdnsal", 2, "dev,num",           [D, LA]),
    (Bdse,   "bdse",   2, "dev,num",           [D, LA]),
    (Bdseal, "bdseal", 2, "dev,num",           [D, LA]),
    (Brdns,  "brdns",  2, "dev,num",           [D, LR]),
    (Brdse,  "brdse",  2, "dev,num",           [D, LR]),
    (L,      "l",      3, "reg,dev,token",     [R, D, S]),
    (Lb,     "lb",     4, "reg,num,token,num", [R, N, S, N]),
    (Lr,     "lr",     4, "reg,dev,num,token", [R, D, N, S]),
    (Ls,     "ls",     4, "reg,dev,num,token", [R, D, N, S]),
    (S,      "s",      3, "dev,token,num",     [D, S, N]),
    (Sb,     "sb",     3, "num,dev,num",       [N, D, N]),

    // Flow Control, Branches and Jumps
    (Bap,    "bap",    4, "num,num,num,num",   [N, N, N, LA]),
    (Bapal,  "bapal",  4, "num,num,num,num",   [N, N, N, LA]),
    (Bapz,   "bapz",   3, "num,num,num",       [N, N, LA]),
    (Bapzal, "bapzal", 3, "num,num,num",       [N, N, LA]),
    (Beq,    "beq",    3, "num,num,num",       [N, N, LA]),
    (Beqal,  "beqal",  3, "num,num,num",       [N, N, LA]),
    (Beqz,   "beqz",   2, "num,num",           [N, LA]),
    (Beqzal, "beqzal", 2, "num,num",           [N, LA]),
    (Bge,    "bge",    3, "num,num,num",       [N, N, LA]),
    (Bgeal,  "bgeal",  3, "num,num,num",       [N, N, LA]),
    (Bgez,   "bgez",   2, "num,num",           [N, LA]),
    (Bgezal, "bgezal", 2, "num,num",           [N, LA]),
    (Bgt,    "bgt",    3, "num,num,num",       [N, N, LA]),
    (Bgtal,  "bgtal",  3, "num,num,num",       [N, N, LA]),
    (Bgtz,   "bgtz",   2, "num,num",           [N, LA]),
    (Bgtzal, "bgtzal", 2, "num,num",           [N, LA]),
    (Ble,    "ble",    3, "num,num,num",       [N, N, LA]),
    (Bleal,  "bleal",  3, "num,num,num",       [N, N, LA]),
    (Blez,   "blez",   2, "num,num",           [N, LA]),
    (Blezal, "blezal", 2, "num,num",           [N, LA]),
    (Blt,    "blt",    3, "num,num,num",       [N, N, LA]),
    (Bltal,  "bltal",  3, "num,num,num",       [N, N, LA]),
    (Bltz,   "bltz",   2, "num,num",           [N, LA]),
    (Bltzal, "bltzal", 2, "num,num",           [N, LA]),
    (Bna,    "bna",    4, "num,num,num,num",   [N, N, N, LA]),
    (Bnaal,  "bnaal",  4, "num,num,num,num",   [N, N, N, LA]),
    (Bnaz,   "bnaz",   3, "num,num,num",       [N, N, LA]),
    (Bnazal, "bnazal", 3, "num,num,num",       [N, N, LA]),
    (Bne,    "bne",    3, "num,num,num",       [N, N, LA]),
    (Bneal,  "bneal",  3, "num,num,num",       [N, N, LA]),
    (Bnez,   "bnez",   2, "num,num",           [N, LA]),
    (Bnezal, "bnezal", 2, "num,num",           [N, LA]),

    (Brap,   "brap",   4, "num,num,num,num",   [N, N, N, LR]),
    (Brapz,  "brapz",  3, "num,num,num",       [N, N, LR]),
    (Breq,   "breq",   3, "num,num,num",       [N, N, LR]),
    (Breqz,  "breqz",  2, "num,num",           [N, LR]),
    (Brge,   "brge",   3, "num,num,num",       [N, N, LR]),
    (Brgez,  "brgez",  2, "num,num",           [N, LR]),
    (Brgt,   "brgt",   3, "num,num,num",       [N, N, LR]),
    (Brgtz,  "brgtz",  2, "num,num",           [N, LR]),
    (Brle,   "brle",   3, "num,num,num",       [N, N, LR]),
    (Brlez,  "brlez",  2, "num,num",           [N, LR]),
    (Brlt,   "brlt",   3, "num,num,num",       [N, N, LR]),
    (Brltz,  "brltz",  2, "num,num",           [N, LR]),
    (Brna,   "brna",   4, "num,num,num,num",   [N, N, N, LR]),
    (Brnaz,  "brnaz",  3, "num,num,num",       [N, N, LR]),
    (Brne,   "brne",   3, "num,num,num",       [N, N, LR]),
    (Brnez,  "brnez",  2, "num,num",           [N, LR]),
    (J,      "j",      1, "num",               [LR]),
    (Jal,    "jal",    1, "num",               [LR]),
    (Jr,     "jr",     1, "num",               [LR]),

    // Variable Selection
    (Sap,    "sap",    4, "reg,num,num,num",   [R, N, N, N]),
    (Sapz,   "sapz",   3, "reg,num,num",       [R, N, N]),
    (Sdns,   "sdns",   2, "reg,dev",           [R, D]),
    (Sdse,   "sdse",   2, "reg,dev",           [R, D]),
    (Select, "select", 4, "reg,num,num,num",   [R, N, N, N]),
    (Seq,    "seq",    3, "reg,num,num",       [R, N, N]),
    (Seqz,   "seqz",   2, "reg,num",           [R, N]),
    (Sge,    "sge",    3, "reg,num,num",       [R, N, N]),
    (Sgez,   "sgez",   2, "reg,num",           [R, N]),
    (Sgt,    "sgt",    3, "reg,num,num",       [R, N, N]),
    (Sgtz,   "sgtz",   2, "reg,num",           [R, N]),
    (Sle,    "sle",    3, "reg,num,num",       [R, N, N]),
    (Slez,   "slez",   2, "reg,num",           [R, N]),
    (Slt,    "slt",    3, "reg,num,num",       [R, N, N]),
    (Sltz,   "sltz",   2, "reg,num",           [R, N]),
    (Sna,    "sna",    4, "reg,num,num,num",   [R, N, N, N]),
    (Snaz,   "snaz",   3, "reg,num,num",       [R, N, N]),
    (Sne,    "sne",    3, "reg,num,num",       [R, N, N]),
    (Snez,   "snez",   2, "reg,num",           [R, N]),

    // Mathematical Operations
    (Abs,    "abs",    2, "reg,num",           [R, N]),
    (Acos,   "acos",   2, "reg,num",           [R, N]),
    (Add,    "add",    3, "reg,num,num",       [R, N, N]),
    (Asin,   "asin",   2, "reg,num",           [R, N]),
    (Atan,   "atan",   2, "reg,num",           [R, N]),
    (Ceil,   "ceil",   2, "reg,num",           [R, N]),
    (Cos,    "cos",    2, "reg,num",           [R, N]),
    (Div,    "div",    3, "reg,num,num",       [R, N, N]),
    (Exp,    "expr",   2, "reg,num",           [R, N]),
    (Floor,  "floor",  2, "reg,num",           [R, N]),
    (Log,    "log",    2, "reg,num",           [R, N]),
    (Max,    "max",    3, "reg,num,num",       [R, N, N]),
    (Min,    "min",    3, "reg,num,num",       [R, N, N]),
    (Mod,    "mod",    3, "reg,num,num",       [R, N, N]),
    (Mul,    "mul",    3, "reg,num,num",       [R, N, N]),
    (Rand,   "rand",   1, "reg",               [R]),
    (Round,  "round",  2, "reg,num",           [R, N]),
    (Sin,    "sin",    2, "reg,num",           [R, N]),
    (Sqrt,   "sqrt",   2, "reg,num",           [R, N]),
    (Sub,    "sub",    3, "reg,num,num",       [R, N, N]),
    (Tan,    "tan",    2, "reg,num",           [R, N]),
    (Trunc,  "trunc",  2, "reg,num",           [R, N]),

    // Logic
    (And,    "and",    3, "reg,num,num",       [R, N, N]),
    (Nor,    "nor",    3, "reg,num,num",       [R, N, N]),
    (Or,     "or",     3, "reg,num,num",       [R, N, N]),
    (Xor,    "xor",    3, "reg,num,num",       [R, N, N]),

    // Stack
    (Peek,   "peek",   1, "reg",               [R]),
    (Pop,    "pop",    1, "reg",               [R]),
    (Push,   "push",   1, "reg",               [R]),

    // Misc
    (Alias,  "alias",  2, "token,dev/reg",     [S, DoR]),
    (Define, "define", 2, "token,num",         [S, N]),
    (Hcf,    "hcf",    0, "null",              []),
    (Move,   "move",   2, "reg,num",           [R, N]),
    (Sleep,  "sleep",  1, "num",               [N]),
    (Yield,  "yield",  0, "null",              []),
);

impl Stmt {
    pub fn get_arg(&self, i: usize) -> Option<&Arg> {
        self.args().get(i)
    }

    pub fn iter_args(&self) -> impl Iterator<Item = &Arg> {
        self.args().iter()
    }

    pub fn iter_args_mut(&mut self) -> impl Iterator<Item = &mut Arg> {
        self.args_mut().iter_mut()
    }

    pub fn reduce_args(&mut self, aliases: &Aliases) -> MipsResult<()> {
        for arg in self.iter_args_mut() {
            if let Some(_key) = arg.as_alias() {
                *arg = arg.clone().reduce(aliases)?;
                // if !mips.present_aliases.contains(key) {
                    // *arg = arg.clone().reduce(aliases)?;
                // }
            }
        }
        Ok(())
    }
}

// impl<'i> MipsNode<'i> for Stmt {
//     fn as_reg_base(&self) -> Option<RegBase> {
//         None
//     }

//     fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> {
//         None
//     }

//     fn as_alias(&self) -> Option<&Arg::String(..)> {
//         None
//     }

//     fn reduce(mut self, mips: &Mips) -> MipsResult<Self> {
//         for arg in self.iter_args_mut() {
//             if let Some(key) = arg.as_alias() {
//                 if !mips.present_aliases.contains(key) {
//                     *arg = arg.clone().reduce(mips)?;
//                 }
//             }
//         }
//         Ok(self)
//     }
// }

// impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Stmt {
//     type Output = Self;

//     const RULE: Rule = Rule::item;

//     fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
//         match pair.as_rule() {
//             Rule::empty => Ok(Self::Empty([])),
//             Rule::tag => {
//                 let tag = pair.as_str().to_owned();
//                 Ok(Self::Tag([tag.into()]))
//             }
//             Rule::stmt => {
//                 let mut pairs = pair.into_inner();
//                 let instr = pairs.next_pair().unwrap().as_str();
//                 let arg_pairs = pairs.collect();
//                 Ok(Self::try_from_name(instr, arg_pairs).unwrap())
//             }
//             _ => unreachable!("{:?}", pair),
//         }
//     }
// }
