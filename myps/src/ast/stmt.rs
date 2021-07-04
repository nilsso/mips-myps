use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};
use mips::{MipsResult, Mips};

use crate::ast::{Expr, Lv, Rv, BinaryOp, IntoMips};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Stmt {
    LvRvAsn(Vec<Lv>, Vec<Rv>),
    SelfAsn {
        op: BinaryOp,
        lhs: Lv,
        rhs: Expr
    },
    Mips(mips::ast::Stmt),
    Empty,
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Stmt {
    type Output = Self;

    const RULE: Rule = Rule::stmt;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::stmt => pair.only_inner().unwrap().try_into_ast(),
            Rule::stmt_lvrv_asn => {
                let pairs = pair.into_inner();
                let (lv_pairs, rv_pairs) =
                    pairs.partition::<Vec<Pair>, _>(|pair| matches!(pair.as_rule(), Rule::lv));
                if lv_pairs.len() != rv_pairs.len() {
                    return Err(MypsError::lv_rv_asn_wrong_num(
                        lv_pairs.len(),
                        rv_pairs.len(),
                    ));
                }
                let lvs = lv_pairs
                    .into_iter()
                    .map(Lv::try_from_pair)
                    .collect::<MypsResult<Vec<_>>>()
                    .unwrap();
                let rvs = rv_pairs
                    .into_iter()
                    .map(Rv::try_from_pair)
                    .collect::<MypsResult<Vec<_>>>()
                    .unwrap();
                for (lv, rv) in lvs.iter().zip(rvs.iter()) {
                    if matches!(rv, Rv::Dev(..)) && !matches!(lv, Lv::Var{..}) {
                        return Err(MypsError::lv_rv_asn_wrong_lv_for_rv_dev(lv, rv));
                    }
                }
                Ok(Self::LvRvAsn(lvs, rvs))
            }
            Rule::stmt_self_asn => {
                let mut pairs = pair.into_inner();
                println!("{:#?}", pairs);
                let lhs = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let op_pair = pairs.next_pair().unwrap();
                let op = match op_pair.as_rule() {
                    Rule::op_s_add => BinaryOp::Add,
                    Rule::op_s_sub => BinaryOp::Sub,
                    Rule::op_s_mul => BinaryOp::Mul,
                    Rule::op_s_div => BinaryOp::Div,
                    Rule::op_s_rem => BinaryOp::Rem,
                    _ => return Err(MypsError::pair_wrong_rule("a self-assign operator", op_pair)),
                };
                let rhs = pairs.final_pair().unwrap().try_into_ast().unwrap();
                Ok(Self::SelfAsn { op, lhs, rhs })
            },
            Rule::stmt_mips => {
                use pest::Parser;

                let mips_str = itertools::join(
                    pair.into_inner()
                        .map(|pair| pair.as_str().to_owned())
                        .collect::<Vec<_>>(),
                    " ",
                );
                let stmt_pair = mips::MipsParser::parse(mips::Rule::line, &mips_str)
                    .unwrap()
                    .only_pair()
                    .unwrap();
                let stmt = mips::ast::Stmt::try_from_pair(stmt_pair).unwrap();
                Ok(Self::Mips(stmt))
            }
            Rule::stmt_empty => Ok(Stmt::Empty),
            _ => Err(MypsError::pair_wrong_rule("a statement", pair)),
        }
    }
}

impl<'i> IntoMips<'i> for Stmt {
    type Output = Vec<mips::ast::Stmt>;

    fn try_into_mips(self, mips: &Mips) -> MipsResult<Self::Output> {
        use mips::ast::Stmt;

        match self {
            Self::LvRvAsn(lvs, rvs) => {
                unreachable!();
            },
            Self::SelfAsn { op, lhs, rhs } => {
                use crate::ast::BinaryOp;

                // let rhs = rhs;

                // match op {
                //     BinaryOp::Add => Stmt::Add([
                // }
                unreachable!();
            },
            Self::Mips(stmt) => {
                Ok(vec![stmt])
            },
            Self::Empty => {
                Ok(vec![Stmt::Empty([])])
            },
        }
    }
}
