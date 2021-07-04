use std::convert::TryInto;

use itertools::join;

use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};

use crate::ast::{Num, Expr};
use crate::{MypsError, MypsParser, MypsResult, Pair, Pairs, Rule};

// macro_rules! validate_arg {
//     (@parse $arg_pair_iter:ident) => {
//         $arg_pair_iter
//             .next()
//             .unwrap()
//             .try_into_ast::<Rv>()
//             .unwrap()
//     };
//     (Num, $disp:literal, $expected:literal, $args_str:ident, $arg_pair_iter:ident) => {{
//         let rv = validate_arg!(@parse $arg_pair_iter);
//         match rv {
//             Rv::Var(..) | Rv::Expr(..) => rv,
//             Rv::Dev(..) => {
//                 return Err(MypsError::num_fun_wrong_args($disp, $expected, &$args_str))
//             },
//         }
//     }};
//     (Dev, $disp:literal, $expected:literal, $args_str:ident, $arg_pair_iter:ident) => {{
//         let rv = validate_arg!(@parse $arg_pair_iter);
//         match rv {
//             Rv::Var(..) | Rv::Dev(..) => rv,
//             Rv::Expr(..) => {
//                 return Err(MypsError::num_fun_wrong_args($disp, $expected, &$args_str))
//             },
//         }
//     }};
// }

macro_rules! def_func {
    ($(
        ($name:ident, $n_args:literal, $disp:literal)
    ),*$(,)*) => {
        #[derive(Clone, Debug)]
        pub enum Func {
            $(
                $name([Num; $n_args]),
            )*
        }

        impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Func {
            type Output = Self;

            const RULE: Rule = Rule::num_func;

            fn try_from_pair(pair: Pair) -> MypsResult<Self::Output> {
                let mut pairs = pair.into_inner();
                let name_str = pairs.next_pair().unwrap().as_str();
                let args = pairs.map(Num::try_from_pair).collect::<MypsResult<Vec<Num>>>().unwrap();
                match name_str.to_lowercase().as_str() {
                    $(
                        $disp => {
                            if $n_args != args.len() {
                                return Err(MypsError::pairs_wrong_num($n_args, args.len()));
                            }
                            let args: [Num; $n_args] = args.try_into().unwrap();
                            Ok(Self::$name(args))
                        }
                    )*
                    _ => Err(MypsError::num_func_unknown(name_str)),
                }
            }
        }
    };
}

def_func!(
    // Math
    // (Abs,   1
    // (Acos,  1
    // (Asin,  1
    // (Ceil,  1
    // (Cos,   1
    // (Exp,   1
    // (Floor, 1
    // (Log,   1
    // (Max,   2
    // (Min,   2
    // (Rand,  1
    // (Round, 1
    // (Sin,   1
    // (Sqrt,  1
    // (Tan,   1
    (Trunc, 1, "trunc"),
    // Stack
    // (Peek,  0
    // (Pop,   0
);

