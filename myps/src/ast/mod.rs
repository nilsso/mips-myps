// mod num;
// pub use num::Num;

mod var;
pub use var::Var;

mod lv;
pub use lv::Lv;

mod rv;
pub use rv::Rv;

mod expr;
pub use expr::{Expr, UnaryOp, BinaryOp};

mod branch;
pub use branch::Branch;

mod stmt;
pub use stmt::Stmt;

mod unit;
pub use unit::{Unit, Line};

use mips::ast::MipsNode;

pub trait MypsNode
where
    Self: std::fmt::Debug,
{
    type MipsOutput;

    fn into_mips_node(self) -> Self::MipsOutput;
}
