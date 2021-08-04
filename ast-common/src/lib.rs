#![feature(stmt_expr_attributes)]
#![allow(unused_imports)]
use std::{fmt, fmt::Display};

use pest::iterators::Pair;
use pest::{Parser, RuleType};

use ast_traits::{AstError, AstNode};
// use crate::ast::{MipsNode, RegBase};
// use crate::{Mips, MipsError, MipsParser, MipsResult, Pair, Rule};

mod constants;

// pub trait AstCommonRule: RuleType {
//     // Note that these rules aren't used for anything.
//     // Rust doesn't have associated constant pattern matching, making these useless.
//     const RULE_BATCH_MODE: Self;
//     const RULE_REAGENT_MODE: Self;
// }
//
// #[derive(Copy, Clone, Debug)]
// pub enum ModeRepr {
//     Int,
//     Str,
// }
//
// #[inline]
// fn get_repr(int_s: &'static str, pair_s: &str) -> ModeRepr {
//     if int_s == pair_s {
//         ModeRepr::Int
//     } else {
//         ModeRepr::Str
//     }
// }
//
// #[inline]
// fn repr_disp(repr: &ModeRepr, int_repr: &'static str, str_repr: &'static str) -> &'static str {
//     if matches!(repr, ModeRepr::Int) {
//         int_repr
//     } else {
//         str_repr
//     }
// }
//
// #[derive(Copy, Clone, Debug)]
// pub enum BatchMode {
//     Avg(ModeRepr),
//     Sum(ModeRepr),
//     Min(ModeRepr),
//     Max(ModeRepr),
// }
//
// impl<'i, R: AstCommonRule, P: Parser<R>, E: AstError<R>> AstNode<'i, R, P, E> for BatchMode {
//     type Output = Self;
//
//     const RULE: R = R::RULE_BATCH_MODE;
//
//     fn try_from_pair(pair: Pair<'i, R>) -> Result<Self::Output, E> {
//         println!("try BatchMode from {:?}", pair);
//         let s = pair.as_str();
//         #[rustfmt::skip]
//         match s.to_lowercase().as_str() {
//             "0" | "average" | "avg" => Ok(Self::Avg(get_repr("0", s))),
//             "1" | "sum"             => Ok(Self::Sum(get_repr("1", s))),
//             "2" | "minimum" | "min" => Ok(Self::Min(get_repr("2", s))),
//             "3" | "maximum" | "max" => Ok(Self::Max(get_repr("3", s))),
//             _ => Err(E::pair_wrong_rule("a batch mode", pair)),
//         }
//     }
// }
//
// impl Display for BatchMode {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Self::Avg(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "0", "Average")),
//             Self::Sum(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "1", "Sum")),
//             Self::Min(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "2", "Min")),
//             Self::Max(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "3", "Max")),
//         }
//     }
// }
//
// #[derive(Copy, Clone, Debug)]
// pub enum ReagentMode {
//     Contents(ModeRepr),
//     Required(ModeRepr),
//     Recipe(ModeRepr),
// }
//
// impl<'i, R: AstCommonRule, P: Parser<R>, E: AstError<R>> AstNode<'i, R, P, E> for ReagentMode {
//     type Output = Self;
//
//     const RULE: R = R::RULE_REAGENT_MODE;
//
//     fn try_from_pair(pair: Pair<'i, R>) -> Result<Self::Output, E> {
//         let s = pair.as_str();
//         #[rustfmt::skip]
//         match s.to_lowercase().as_str() {
//             "0" | "contents" | "cns" => Ok(Self::Contents(get_repr("0", s))),
//             "1" | "required" | "rqd" => Ok(Self::Required(get_repr("1", s))),
//             "2" | "recipe"   | "rcp" => Ok(Self::Recipe(get_repr("2", s))),
//             _ => Err(E::pair_wrong_rule("a reagent mode", pair)),
//         }
//     }
// }
//
// impl Display for ReagentMode {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Self::Contents(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "0", "Contents")),
//             Self::Required(mode_repr) => write!(f, "{}", repr_disp(mode_repr, "1", "Required")),
//             Self::Recipe(mode_repr)   => write!(f, "{}", repr_disp(mode_repr, "2", "Recipe")),
//         }
//     }
// }
