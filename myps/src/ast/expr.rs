use lazy_static::lazy_static;
use pest::prec_climber::{Assoc, Operator, PrecClimber};

use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};
use mips::{Mips, MipsResult};

use crate::ast::{IntoMips, Num};
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
            Rule::op_b_or => Ok(Self::Or),
            Rule::op_b_xor => Ok(Self::Xor),
            // Relational
            Rule::op_b_eq => Ok(Self::Eq),
            Rule::op_b_ge => Ok(Self::Ge),
            Rule::op_b_gt => Ok(Self::Gt),
            Rule::op_b_le => Ok(Self::Le),
            Rule::op_b_lt => Ok(Self::Lt),
            Rule::op_b_ne => Ok(Self::Ne),
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

impl_from_primitive!(Expr, Expr::Num, n, { n.into() });

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
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Expr {
    type Output = Self;

    const RULE: Rule = Rule::expr;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::expr => pair.only_inner().unwrap().try_into_ast(),
            Rule::expr_unary => {
                let mut pairs = pair.into_inner();
                let op = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let rhs = pairs.final_pair().unwrap().try_into_ast().unwrap();
                Ok(Expr::unary(op, rhs))
            }
            Rule::expr_binary => Ok(expr_climb(pair.into_inner())),
            Rule::expr_ternary => {
                let mut pairs = pair.into_inner();
                let cond = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let if_t = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let if_f = pairs.final_pair().unwrap().try_into_ast().unwrap();
                Ok(Expr::ternary(cond, if_t, if_f))
            }
            Rule::num_var | Rule::num | Rule::var => Ok(Self::Num(pair.try_into_ast().unwrap())),
            _ => {
                return Err(MypsError::pair_wrong_rule(
                    "an expression or number-like",
                    pair,
                ))
            }
        }
    }
}

impl<'i> IntoMips<'i> for Expr {
    type Output = (usize, mips::ast::Num, Vec<mips::ast::Stmt>);

    fn try_into_mips(self, mips: &Mips) -> MipsResult<Self::Output> {
        match self {
            Self::Num(num) => {
                unimplemented!();
            }
            Self::Unary { op, rhs } => {
                unimplemented!();
            }
            Self::Binary { op, lhs, rhs } => {
                // let a = lhs.try_into_mips(mips);
                unimplemented!();
            }
            Self::Ternary { cond, if_t, if_f } => {
                unimplemented!();
            }
        }
    }
}

// Operator precedence climber
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
