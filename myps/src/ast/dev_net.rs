
use crate::ast::Expr;

#[derive(Clone, Debug)]
pub enum DevNet {
    Lit(Box<Expr>),
    Alias(String),
}

