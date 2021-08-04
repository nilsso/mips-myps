use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};
use mips::MipsResult;

use crate::ast::{BinaryOp, Expr, Lv, Rv, Var};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub enum Stmt {
    Fix(Vec<String>),
    Asn(Lv, Rv),
    SelfAsn { op: BinaryOp, lhs: Var, rhs: Expr },
    Mips(mips::ast::Stmt),
    Empty,
}

impl<'i> AstNode<'i, Rule, MypsParser, MypsError> for Stmt {
    type Output = Self;

    const RULE: Rule = Rule::stmt;

    fn try_from_pair(pair: Pair<'i>) -> MypsResult<Self::Output> {
        match pair.as_rule() {
            Rule::stmt => pair.only_inner().unwrap().try_into_ast(),
            Rule::stmt_fix => {
                let names = pair
                    .into_inner()
                    .map(|pair| pair.as_str().to_owned())
                    .collect();
                Ok(Self::Fix(names))
            }
            Rule::stmt_asn => {
                let mut pairs = pair.into_inner();
                let lv = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let rv = pairs.final_pair().unwrap().try_into_ast().unwrap();
                Ok(Self::Asn(lv, rv))
            }
            Rule::stmt_self_asn => {
                let mut pairs = pair.into_inner();
                let lhs = pairs.next_pair().unwrap().try_into_ast().unwrap();
                let op_pair = pairs.next_pair().unwrap();
                let op = match op_pair.as_rule() {
                    Rule::op_s_add => BinaryOp::Add,
                    Rule::op_s_sub => BinaryOp::Sub,
                    Rule::op_s_mul => BinaryOp::Mul,
                    Rule::op_s_div => BinaryOp::Div,
                    Rule::op_s_rem => BinaryOp::Rem,
                    _ => {
                        return Err(MypsError::pair_wrong_rule(
                            "a self-assign operator",
                            op_pair,
                        ))
                    }
                };
                let rhs = pairs.final_pair().unwrap().try_into_ast().unwrap();
                Ok(Self::SelfAsn { op, lhs, rhs })
            }
            Rule::stmt_mips => {
                use pest::Parser;

                let mips_str = itertools::join(
                    pair.into_inner()
                        .map(|pair| pair.as_str().to_owned())
                        .collect::<Vec<_>>(),
                    " ",
                );
                let stmt_pair = mips::MipsParser::parse(mips::Rule::stmt, &mips_str)
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
