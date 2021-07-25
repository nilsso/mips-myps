use std::convert::TryInto;

use itertools::join;

use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};
use mips::MipsResult;

use crate::ast::{Expr, Num, Dev};
use crate::{MypsError, MypsParser, MypsResult, Pair, Pairs, Rule};

#[derive(Clone, Debug)]
pub enum Arg {
    Expr(Expr),
    Dev(Dev),
}

impl From<Expr> for Arg {
    fn from(expr: Expr) -> Self {
        Self::Expr(expr)
    }
}

impl From<Dev> for Arg {
    fn from(dev: Dev) -> Self {
        Self::Dev(dev)
    }
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Arg {
    type Output = Self;

    const RULE: Rule = Rule::rv;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::rv => pair.only_inner().unwrap().try_into_ast(),
            Rule::expr => Ok(Self::Expr(pair.try_into_ast().unwrap())),
            Rule::dev => Ok(Self::Dev(pair.try_into_ast().unwrap())),
            // Rule::var => Ok(Self::Var(pair.try_into_ast().unwrap())),
            _ => Err(MypsError::pair_wrong_rule("an r-value (device or expression)", pair)),
        }
    }
}

macro_rules! def_func {
    ($(
        ($name:ident, $n_args:literal, $disp:literal, $expected:literal, [$($arg_kind:ty),*$(,)*])
    ),*$(,)*) => {
        #[derive(Clone, Debug)]
        pub enum Func {
            // Log([Arg; 2]),
            $(
                $name([Arg; $n_args]),
            )*
        }

        impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Func {
            type Output = Self;

            const RULE: Rule = Rule::num_func;

            fn try_from_pair(pair: Pair) -> MypsResult<Self::Output> {
                let mut pairs = pair.into_inner();
                let name_str = pairs.next_pair().unwrap().as_str();
                let arg_pairs = pairs
                    .map(|pair| {
                        if matches!(pair.as_rule(), Rule::rv) {
                            Ok(pair.only_inner().unwrap())
                        } else {
                            Err(MypsError::pair_wrong_rule("an r-value", pair))
                        }
                    })
                    .collect::<MypsResult<Vec<_>>>()
                    .unwrap();
                #[allow(dead_code, unused_variables, unused_mut)]
                match name_str {
                    $(
                        $disp => {
                            if $n_args != arg_pairs.len() {
                                return Err(MypsError::func_args_wrong_num(
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
                                            MypsError::func_args_wrong_kinds(
                                                name_str,
                                                $expected,
                                                &found,
                                            )
                                        })
                                        .unwrap();
                                    ast.into()
                                }),*
                            ];
                            Ok(Self::$name(args))
                        }
                    )*
                    _ => Err(MypsError::func_unknown(name_str)),
                }
            }
        }
    };
}

#[rustfmt::skip]
use crate::ast::{
    Dev as D,
    Expr as E,
};

#[rustfmt::skip]
def_func!(
    (Dns,   1, "dns",   "dev",       [D   ]),
    (Dse,   1, "dse",   "dev",       [D   ]),
    // Math
    (Abs,   1, "abs",   "expr",      [E   ]),
    (Acos,  1, "acos",  "nexpr",     [E   ]),
    (Asin,  1, "asin",  "expr",      [E   ]),
    (Ceil,  1, "ceil",  "expr",      [E   ]),
    (Cos,   1, "cos",   "expr",      [E   ]),
    (Exp,   1, "exp",   "expr",      [E   ]),
    (Floor, 1, "floor", "expr",      [E   ]),
    (Log,   2, "log",   "expr",      [E, E]), // custom
    (Ln,    1, "ln",    "expr",      [E   ]), // rename for Mips.log
    (Max,   2, "max",   "expr,expr", [E, E]),
    (Min,   2, "min",   "expr,expr", [E, E]),
    (Rand,  0, "rand",  "null",      [    ]),
    (Round, 1, "round", "expr",      [E   ]),
    (Sin,   1, "sin",   "expr",      [E   ]),
    (Sqrt,  1, "sqrt",  "expr",      [E   ]),
    (Tan,   1, "tan",   "expr",      [E   ]),
    (Trunc, 1, "trunc", "expr",      [E   ]),
    // Stack
    (Peek,  0, "peek",  "null",      [    ]),
    (Pop,   0, "pop",   "null",      [    ]),
);

// impl<'i> IntoMips<'i> for Func {
//     type Output = (usize, mips::ast::Num, Vec<mips::ast::Stmt>);

//     fn try_into_mips(self, mips: &Mips) -> MipsResult<Self::Output> {
//     }
// }
