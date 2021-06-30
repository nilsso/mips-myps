use lazy_static::lazy_static;
use pest::prec_climber::{Assoc, Operator, PrecClimber};

use ast_traits::{AstNode, AstPair, AstPairs, IntoAst, AstError};
use mips::ast::Unit;

// use crate::ast::MypsNode;
use crate::{MypsError, MypsParser, MypsResult, Pair, Pairs, Rule};
use crate::ast::{Rv, MypsNode};

#[rustfmt::skip]
#[derive(Clone, Debug)]
pub enum UnaryOp { Inv, Not, }

// impl MypsNode for UnaryOp {
    // type MipsOutput = Vec<
// }

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
    Add, Sub, Mul, Div, Rem,
    // Logical
    And, Or, Xor,
    // Relational
    EQ, GE, GT, LE, LT, NE,
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for BinaryOp {
    type Output = Self;

    const RULE: Rule = Rule::op_b;

    fn try_from_pair(pair: Pair) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::op_b_add => Ok(Self::Add),
            Rule::op_b_sub => Ok(Self::Sub),
            Rule::op_b_mul => Ok(Self::Mul),
            Rule::op_b_div => Ok(Self::Div),
            Rule::op_b_rem => Ok(Self::Rem),
            Rule::op_b_and => Ok(Self::And),
            Rule::op_b_or  => Ok(Self::Or),
            Rule::op_b_xor => Ok(Self::Xor),
            Rule::op_b_eq  => Ok(Self::EQ),
            Rule::op_b_ge  => Ok(Self::GE),
            Rule::op_b_gt  => Ok(Self::GT),
            Rule::op_b_le  => Ok(Self::LE),
            Rule::op_b_lt  => Ok(Self::LT),
            Rule::op_b_ne  => Ok(Self::NE),
            _ => Err(MypsError::pair_wrong_rule("a binary operator", pair)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Rv(Rv),
    Unary {
        op: UnaryOp,
        rhs: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Ternary {
        cond: Box<Expr>,
        if_t: Box<Expr>,
        if_f: Box<Expr>,
    },
}

impl Expr {
    pub fn unary(op: UnaryOp, rhs: Expr) -> Self {
        let rhs = Box::new(rhs);
        Self::Unary { op, rhs }
    }

    pub fn binary(op: BinaryOp, lhs: Expr, rhs: Expr) -> Self {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);
        Self::Binary { op, lhs, rhs }
    }

    pub fn ternary(cond: Expr, if_t: Expr, if_f: Expr) -> Self {
        let cond = Box::new(cond);
        let if_t = Box::new(if_t);
        let if_f = Box::new(if_f);
        Self::Ternary { cond, if_t, if_f }
    }
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Expr {
    type Output = Self;

    const RULE: Rule = Rule::expr;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::expr_unary => {
                let mut pairs = pair.into_inner();
                let op = pairs.next_pair()?.try_into_ast()?;
                let rhs = pairs.final_pair()?.try_into_ast()?;
                Ok(Expr::unary(op, rhs))
            }
            Rule::expr_binary => {
                Ok(expr_climb(pair.into_inner()))
            },
            Rule::expr_ternary => {
                let mut pairs = pair.into_inner();
                let cond = pairs.next_pair()?.try_into_ast()?;
                let if_t = pairs.next_pair()?.try_into_ast()?;
                let if_f = pairs.final_pair()?.try_into_ast()?;
                Ok(Expr::ternary(cond, if_t, if_f))
            }
            Rule::rv => {
                Ok(Self::Rv(pair.try_into_ast()?))
            },
            _ => return Err(MypsError::pair_wrong_rule("an r-value expression", pair)),

        }
    }
}

// Operator precedence climber
lazy_static! {
    static ref CLIMBER: PrecClimber<Rule> = PrecClimber::new(vec![
        // Logical
        Operator::new(Rule::op_b_or, Assoc::Left),
        Operator::new(Rule::op_b_xor, Assoc::Left),
        Operator::new(Rule::op_b_and, Assoc::Left),
        // Relational
        Operator::new(Rule::op_b_eq, Assoc::Left),
        Operator::new(Rule::op_b_ge, Assoc::Left),
        Operator::new(Rule::op_b_gt, Assoc::Left),
        Operator::new(Rule::op_b_le, Assoc::Left),
        Operator::new(Rule::op_b_lt, Assoc::Left),
        Operator::new(Rule::op_b_ne, Assoc::Left),
        // Numerical
        Operator::new(Rule::op_b_add, Assoc::Left),
        Operator::new(Rule::op_b_sub, Assoc::Left),
        Operator::new(Rule::op_b_rem, Assoc::Left),
        Operator::new(Rule::op_b_div, Assoc::Left),
        Operator::new(Rule::op_b_mul, Assoc::Left),
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

