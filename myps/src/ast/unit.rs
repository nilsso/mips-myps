use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};

use mips::ast as mips_ast;

// use crate::ast::Stmt;
use crate::ast::{Expr, Lv};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Unit {
    LvRvAsn(Vec<Lv>, Vec<Expr>),
    Mips(mips_ast::Unit),
    Empty,
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Unit {
    type Output = Self;

    const RULE: Rule = Rule::unit;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::unit => pair.only_inner().unwrap().try_into_ast(),
            Rule::unit_lvrv_asn => {
                let pairs = pair.into_inner();
                let (lv_pairs, expr_pairs) = pairs
                    .partition::<Vec<Pair>, _>(|pair| matches!(pair.as_rule(), Rule::lv));
                if lv_pairs.len() != expr_pairs.len() {
                    return Err(MypsError::lv_expr_asn_wrong_num(lv_pairs.len(), expr_pairs.len()));
                }
                let lvs = lv_pairs
                    .into_iter()
                    .map(Lv::try_from_pair)
                    .collect::<MypsResult<Vec<_>>>().unwrap();
                let exprs = expr_pairs
                    .into_iter()
                    .map(Expr::try_from_pair)
                    .collect::<MypsResult<Vec<_>>>().unwrap();
                for (lv, expr) in lvs.iter().zip(exprs.iter()) {
                    // TODO: Need different structure for right-hand-side things
                    // needs to encompass expressions and devices
                    // Gonna rename Rv to Value and let Rv be the enum to store either
                    // a device or a value
                }
                Ok(Self::LvRvAsn(lvs, exprs))
            }
            Rule::unit_mips => {
                let mut pairs = pair.into_inner();
                let name = pairs.next_pair().unwrap().as_str().to_owned();
                unreachable!();
                // let args =
                // pairs.map(Expr::try_from_pair).collect::<MypsResult<Vec<Expr>>>().unwrap();
                // Ok(Self::Func(name, args))
            }
            Rule::empty => Ok(Unit::Empty),
            _ => Err(MypsError::pair_wrong_rule("a unit", pair)),
        }
    }
}

// impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Line {
//     type Output = (usize, Unit);

//     const RULE: Rule = Rule::unit;

//     fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
//         match pair.as_rule() {
//             Rule::line => {
//                 let mut pairs = pair.into_inner();
//                 let spaces = pairs.next_rule(Rule::indent, "an indent")?
//                     .as_str().len();
//                 let unit_pair = pairs.next_rule(Rule::unit, "a unit")?.only_inner()?;
//                 let comment_opt = {
//                     let comment_pair = pairs.final_rule(Rule::comment, "a comment")?;
//                     let s = comment_pair.as_str();
//                     (s.len() > 0).then_some(s.to_owned())
//                 };
//                 let unit = match unit_pair.as_rule() {
//                     Rule::branch => Unit::Branch(unit_pair.try_into_ast()?, comment_opt),
//                     Rule::stmt => Unit::Stmt(unit_pair.try_into_ast()?, comment_opt),
//                     Rule::empty => Unit::Empty(comment_opt),
//                     _ => return Err(MypsError::pair_wrong_rule("a branch, statement or empty unit", unit_pair)),
//                 };
//                 Ok((spaces, unit))
//             },
//             _ => Err(MypsError::pair_wrong_rule("a unit line", pair)),
//         }
//     }
// }

// #[derive(Clone, Debug)]
// pub struct Line;

// impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Line {
//     type Output = (usize, Self);

//     const RULE: Rule = Rule::line;

//     fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
//         match pair.as_rule() {
//             Rule::line => {
//                 let mut pairs = pair.into_inner();
//                 let spaces = pairs.next_rule(Rule::indent, "an indent")?
//                     .as_str().len();
//                 let unit = pairs.next_rule(Rule::unit, "a unit")?.try_into_ast()?;
//                 // pairs.next();
//                 // let unit = Unit::Empty;
//                 let comment_opt = {
//                     let comment_pair = pairs.next_rule(Rule::comment, "a comment")?;
//                     let s = comment_pair.as_str();
//                     (s.len() > 0).then_some(s.to_owned())
//                 };
//                 Ok((spaces, Self(unit, comment_opt)))
//             },
//             _ => Err(MypsError::pair_wrong_rule("an line", pair)),
//         }
//     }
// }
