use lazy_static::lazy_static;
use pest::prec_climber::{Assoc, Operator, PrecClimber};

use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};
use mips::MipsResult;

use crate::ast::Num;
use crate::{MypsError, MypsParser, MypsResult, Pair, Pairs, Rule};

#[rustfmt::skip]
#[derive(Clone, Debug)]
pub enum UnaryOp { Inv, Not, }

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for UnaryOp {
    type Output = Self;

    const RULE: Rule = Rule::op_u;

    fn try_from_pair(pair: Pair) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::op_u_inv => Ok(Self::Inv),
            Rule::op_u_not => Ok(Self::Not),
            _ => Err(MypsError::pair_wrong_rule("a unary operator", pair)),
        }
    }
}

#[rustfmt::skip]
#[derive(Clone, Debug)]
pub enum BinaryOp {
    // Numerical
    Add, Sub, Mul, Div, Rem, Pow,
    // Logical
    And, Nor, Or, Xor,
    // Relational
    Eq, Ge, Gt, Le, Lt, Ne,
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for BinaryOp {
    type Output = Self;

    const RULE: Rule = Rule::op_b;

    fn try_from_pair(pair: Pair) -> MypsResult<Self::Output> {
        #[rustfmt::skip]
        match pair.as_rule() {
            // Numerical
            Rule::op_b_add => Ok(Self::Add),
            Rule::op_b_sub => Ok(Self::Sub),
            Rule::op_b_mul => Ok(Self::Mul),
            Rule::op_b_div => Ok(Self::Div),
            Rule::op_b_rem => Ok(Self::Rem),
            Rule::op_b_pow => Ok(Self::Pow),
            // Logical
            Rule::op_b_and => Ok(Self::And),
            Rule::op_b_or  => Ok(Self::Or),
            Rule::op_b_xor => Ok(Self::Xor),
            // Relational
            Rule::op_b_eq  => Ok(Self::Eq),
            Rule::op_b_ge  => Ok(Self::Ge),
            Rule::op_b_gt  => Ok(Self::Gt),
            Rule::op_b_le  => Ok(Self::Le),
            Rule::op_b_lt  => Ok(Self::Lt),
            Rule::op_b_ne  => Ok(Self::Ne),
            _ => Err(MypsError::pair_wrong_rule("a binary operator", pair)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Num(Num),
    Unary { op: UnaryOp, rhs: Num },
    Binary { op: BinaryOp, lhs: Num, rhs: Num },
    Ternary { cond: Num, if_t: Num, if_f: Num },
}

fn bool_to_float(b: bool) -> f64 {
    if b {
        1_f64
    } else {
        0_f64
    }
}

impl Expr {
    pub fn unary(op: UnaryOp, rhs: Expr) -> Self {
        let rhs = rhs.into();
        Self::Unary { op, rhs }
    }

    pub fn binary(op: BinaryOp, lhs: Expr, rhs: Expr) -> Self {
        let lhs = lhs.into();
        let rhs = rhs.into();
        Self::Binary { op, lhs, rhs }
    }

    pub fn ternary(cond: Expr, if_t: Expr, if_f: Expr) -> Self {
        let cond = cond.into();
        let if_t = if_t.into();
        let if_f = if_f.into();
        Self::Ternary { cond, if_t, if_f }
    }

    pub fn simplify(self) -> Self {
        match self {
            Self::Unary { op, rhs } => {
                let rhs = rhs.simplify();
                match rhs {
                    Num::Lit(n) => {
                        let n = match op {
                            UnaryOp::Inv => -n,
                            UnaryOp::Not => bool_to_float(n == 0_f64),
                        };
                        n.into()
                    }
                    rhs => Self::Unary { op, rhs },
                }
            }
            Self::Binary { op, lhs, rhs } => {
                let lhs = lhs.simplify();
                let rhs = rhs.simplify();
                match (lhs, rhs) {
                    (Num::Lit(l), Num::Lit(r)) => {
                        #[rustfmt::skip]
                        let n = match op {
                            BinaryOp::Add => l + r,
                            BinaryOp::Sub => l - r,
                            BinaryOp::Mul => l * r,
                            BinaryOp::Div => l / r,
                            BinaryOp::Rem => l % r,
                            BinaryOp::Pow => l.powf(r),
                            // Logical
                            BinaryOp::And => bool_to_float((l != 0_f64) & (r != 0_f64)),
                            BinaryOp::Nor => unimplemented!(),
                            BinaryOp::Or  => bool_to_float((l != 0_f64) | (r != 0_f64)),
                            BinaryOp::Xor => bool_to_float((l != 0_f64) ^ (r != 0_f64)),
                            // Relational
                            BinaryOp::Eq  => bool_to_float(l == r),
                            BinaryOp::Ge  => bool_to_float(l >= r),
                            BinaryOp::Gt  => bool_to_float(l >  r),
                            BinaryOp::Le  => bool_to_float(l <= r),
                            BinaryOp::Lt  => bool_to_float(l <  r),
                            BinaryOp::Ne  => bool_to_float(l != r),
                        };
                        n.into()
                    }
                    (lhs, rhs) => Self::Binary { op, lhs, rhs },
                }
            }
            Self::Ternary { cond, if_t, if_f } => {
                let cond = cond.simplify();
                let if_t = if_t.simplify();
                let if_f = if_f.simplify();
                match (cond, if_t, if_f) {
                    (Num::Lit(c), Num::Lit(t), Num::Lit(f)) => {
                        if c != 0_f64 {
                            t.into()
                        } else {
                            f.into()
                        }
                    }
                    (cond, if_t, if_f) => Self::Ternary { cond, if_t, if_f },
                }
            }
            _ => self,
        }
    }
}

impl_from_primitive!(Expr, Expr::Num, n, { n.into() });

impl From<Num> for Expr {
    fn from(num: Num) -> Self {
        Self::Num(num)
    }
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Expr {
    type Output = Self;

    const RULE: Rule = Rule::expr;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        let expr = match pair.as_rule() {
            Rule::expr => pair.only_inner().unwrap().try_into_ast().unwrap(),
            Rule::expr_unary => {
                let mut pairs = pair.into_inner();
                let op = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let rhs = pairs.final_pair().unwrap().try_into_ast().unwrap();
                Expr::unary(op, rhs)
            }
            Rule::expr_binary => expr_climb(pair.into_inner()),
            Rule::expr_ternary => {
                let mut pairs = pair.into_inner();
                let cond = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let if_t = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let if_f = pairs.final_pair().unwrap().try_into_ast().unwrap();
                Expr::ternary(cond, if_t, if_f)
            }
            Rule::num_var | Rule::num | Rule::var => Self::Num(pair.try_into_ast().unwrap()),
            _ => {
                return Err(MypsError::pair_wrong_rule(
                    "an expression or number-like",
                    pair,
                ))
            }
        };
        Ok(expr.simplify())
    }
}

#[rustfmt::skip]
use Rule::{
    // Numerical
    op_b_add as Add,
    op_b_sub as Sub,
    op_b_mul as Mul,
    op_b_div as Div,
    op_b_rem as Rem,
    op_b_pow as Pow,
    // Logical
    op_b_and as And,
    op_b_or  as Or,
    op_b_xor as Xor,
    // Relational
    op_b_eq  as Eq,
    op_b_ge  as Ge,
    op_b_gt  as Gt,
    op_b_le  as Le,
    op_b_lt  as Lt,
    op_b_ne  as Ne,
};

macro_rules! Op {
    ($rule:path, L) => {
        Operator::new($rule, Assoc::Left)
    };
    ($rule:path, R) => {
        Operator::new($rule, Assoc::Right)
    };
}

lazy_static! {
    static ref CLIMBER: PrecClimber<Rule> = PrecClimber::new(vec![
        // (from lowest to highest priority)
        Op!(Or,  L) | Op!(Xor, L),
        Op!(And, L),
        Op!(Eq,  L) | Op!(Ne,  L),
        Op!(Ge,  L) | Op!(Gt,  L) | Op!(Le,  L) | Op!(Lt,  L),
        Op!(Add, L) | Op!(Sub, L),
        Op!(Mul, L) | Op!(Div, L) | Op!(Rem, L),
        Op!(Pow, R),
    ]);
}

// Operator precedence climber infix helper
fn infix(lhs: Expr, op_pair: Pair, rhs: Expr) -> Expr {
    Expr::binary(op_pair.into_ast(), lhs, rhs)
}

// Operator precedence climber helper (for binary expressions)
pub fn expr_climb(pairs: Pairs) -> Expr {
    CLIMBER.climb(pairs, Expr::from_pair, infix)
}
