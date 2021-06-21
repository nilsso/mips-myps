use std::{fmt, fmt::Display};

use itertools::join;
use pest::iterators::Pair;

use crate::ast_traits::NextPair;
use crate::mips::ast::{Arg, AstError, AstNode, AstResult, Num, Reg};
use crate::mips::{Alias, Mips, MipsParser, Rule};

macro_rules! def_unit {
    (@try_into_arg $mips:ident, $pair_iter:ident, $ast_kind:ty, $arg_kind:path) => {{
        let pair = $pair_iter.next_pair().unwrap();
        let arg: Arg = <$ast_kind>::try_from_pair($mips, pair).unwrap().into();
        if !matches!(arg, $arg_kind(..)) {
            panic!("Wrong argument {:?}", arg);
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
                pub fn $try(mips: &mut Mips, args: Vec<Pair<Rule>>, comment: Option<String>) -> Self {
                    let mut pair_iter = args.into_iter();
                    let unit = Unit::$kind(
                        [$( def_unit!(@try_into_arg mips, pair_iter, $ast_kind, $arg_kind), )*],
                        comment);
                    if pair_iter.next().is_some() {
                        panic!();
                    }
                    unit
                }
            )*

            pub fn try_from_name(mips: &mut Mips, instr: &str, args: Vec<Pair<Rule>>, comment: Option<String>) -> Self {
                match instr {
                    $(
                        $disp => Self::$try(mips, args, comment),
                    )*
                    _ => panic!(),
                }
            }

            pub fn iter_args(&self) -> impl Iterator<Item = &Arg> {
                match self {
                    $(
                        Self::$kind(args, _) => args.iter(),
                    )*
                    Self::Empty(args, _) => args.iter(),
                }
            }

            pub fn iter_args_mut(&mut self) -> impl Iterator<Item = &mut Arg> {
                match self {
                    $(
                        Self::$kind(args, _) => args.iter_mut(),
                    )*
                    Self::Empty(args, _) => args.iter_mut(),
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
    (Add,   3, "add",   try_add,   [(Reg, Arg::Reg), (Num, Arg::Num), (Num, Arg::Num)]),
    (Sub,   3, "sub",   try_sub,   [(Reg, Arg::Reg), (Num, Arg::Num), (Num, Arg::Num)]),
    (Mul,   3, "mul",   try_mul,   [(Reg, Arg::Reg), (Num, Arg::Num), (Num, Arg::Num)]),
    (Dev,   3, "div",   try_div,   [(Reg, Arg::Reg), (Num, Arg::Num), (Num, Arg::Num)]),

    (Alias, 2, "alias", try_alias, [(String, Arg::String), (Reg, Arg::Reg)]),
    (Move,  2, "move",  try_move,  [(Reg, Arg::Reg), (Num, Arg::Num)]),
);

impl<'i> AstNode<'i, Rule, MipsParser, AstError> for Unit {
    type Output = Self;

    const RULE: Rule = Rule::line;

    fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> AstResult<Self::Output> {
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

            let instr_pair = pairs.next_pair().unwrap();
            let arg_pairs = pairs.collect();

            Self::try_from_name(mips, instr_pair.as_str(), arg_pairs, comment_pair_opt)
        } else {
            Self::Empty([], comment_pair_opt)
        };

        match &unit {
            Unit::Alias([Arg::String(k), Arg::Reg(reg)], _) => {
                mips.aliases.insert(k.clone(), Alias::Reg(reg.clone()));
            }
            _ => {}
        }

        Ok(unit)
    }
}
