use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};
use mips::MipsResult;

use crate::ast::{Dev, Expr, Rv, Var, Func, Lv};
use crate::{MypsError, MypsParser, MypsResult, Pair, Pairs, Rule};

#[derive(Clone, Debug)]
pub enum Num {
    Lit(f64),
    Var(Var),
    Expr(Box<Expr>),
    Func(Box<Func>),
    DevParam {
        dev: Dev,
        param: String,
    },
    DevSlot {
        dev: Dev,
        slot: Box<Num>,
        param: String,
    },
    DevReagent {
        dev: Dev,
        mode: Box<Num>,
        reagent: String,
    },
    NetParam {
        hash: Box<Num>,
        mode: Box<Num>,
        param: String,
    },
}

impl_from_primitive!(Num, Num::Lit, n, { n as f64 });

impl Num {
    pub fn simplify(self) -> Self {
        match self {
            Self::Expr(box expr) => expr.simplify().into(),
            _ => self,
        }
    }
}

impl From<Expr> for Num {
    fn from(expr: Expr) -> Self {
        if let Expr::Num(num) = expr {
            num
        } else {
            Num::Expr(Box::new(expr))
        }
    }
}

// impl Into<mips::ast::Arg> for Num {
//     fn into(self) -> mips::ast::Arg {
//     }
// }

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Num {
    type Output = Self;

    const RULE: Rule = Rule::num_var;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::num_var | Rule::num | Rule::dev_net => pair.only_inner().unwrap().try_into_ast(),
            // A literal number (integer or floating-point)
            Rule::int | Rule::dec => Ok(Self::Lit(pair.as_str().parse().unwrap())),
            // A variable token
            Rule::var => Ok(Self::Var(pair.try_into_ast().unwrap())),
            // A parenthesized expression
            Rule::expr => {
                let expr = pair.try_into_ast().unwrap();
                match expr {
                    Expr::Num(num) => Ok(num),
                    _ => Ok(Self::Expr(Box::new(expr))),
                }
            }
            // A builtin numeric r-value function (e.g. min, pop, sos)
            Rule::num_func => {
                let func = pair.try_into_ast().unwrap();
                Ok(Self::Func(Box::new(func)))
                // let mut pairs = pair.into_inner();
                // let name = pairs.next_pair().unwrap().try_into_ast().unwrap();
                // let args = pairs
                //     .map(Rv::try_from_pair)
                //     .collect::<MypsResult<Vec<Rv>>>()
                //     .unwrap();
                // Ok(Self::Func { name, args })
            }
            //  The value from reading the parameter of a device
            Rule::num_dev_param => {
                let mut pairs = pair.into_inner();
                let dev = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let param = pairs.final_pair().unwrap().try_into_ast().unwrap();
                Ok(Self::DevParam { dev, param })
            }
            //  The value from reading a slot parameter of a device
            Rule::num_dev_slot => {
                let mut pairs = pair.into_inner();
                let dev = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let slot = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let param = pairs.final_pair().unwrap().try_into_ast().unwrap();

                let slot = Box::new(slot);
                Ok(Self::DevSlot { dev, slot, param })
            }
            //  The value from reading a reagent parameter of a device */
            Rule::num_dev_reagent => {
                let mut pairs = pair.into_inner();
                let dev = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let mode = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let reagent = pairs.final_pair().unwrap().try_into_ast().unwrap();

                let mode = Box::new(mode);
                Ok(Self::DevReagent { dev, mode, reagent })
            }
            //  The value from batch-reading the parameter of devices on the data network
            Rule::num_net_param => {
                let mut pairs = pair.into_inner();
                let hash = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let mode = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let param = pairs.final_pair().unwrap().try_into_ast().unwrap();

                let (hash, mode) = (Box::new(hash), Box::new(mode));
                Ok(Self::NetParam { hash, mode, param })
            }
            _ => Err(MypsError::pair_wrong_rule("a number-like", pair)),
        }
    }
}
