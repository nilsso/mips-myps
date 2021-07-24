use ast_traits::{AstNode, IntoAst};
use mips::{Mips, MipsResult, Rule as MipsRule, MipsParser, MipsError};
use mips::ast::MipsNode;

use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};
// use crate::lexer::MypsLexer;

// Helper for From<{primitive}> for Num and Expr
macro_rules! impl_from_primitive {
    ($enum:ty, $variant:path, $n:ident, $expr:block, [$($T:ty),*$(,)*]) => {
        $(
            impl From<$T> for $enum {
                fn from(n: $T) -> Self {
                    let $n = n;
                    $variant($expr)
                }
            }
        )*
    };
    ($enum:ty, $variant:path, $n:ident, $expr:block) => {
        impl_from_primitive!($enum, $variant, $n, $expr, [
            i64, i32, i16, i8,
            u64, u32, u16, u8,
            f64, f32,
            usize, isize,
        ]);
    };
}

mod var;
pub use var::Var;

mod dev;
pub use dev::Dev;

mod num;
pub use num::Num;

mod func;
pub use func::{Arg, Func};

mod expr;
pub use expr::{BinaryOp, Expr, UnaryOp};

// mod dev_net;
// pub use dev_net::DevNet;

mod lv;
pub use lv::Lv;

mod rv;
pub use rv::Rv;

mod stmt;
pub use stmt::Stmt;

// mod unit;
// pub use unit::Unit;

mod branch;
pub use branch::Branch;

mod block;
pub use block::Block;

mod item;
pub use item::{Item, LineItem, LineItemMut};

// use mips::ast::MipsNode;
// use crate::MypsTranslator;

// pub trait MypsNode
// where
//     Self: std::fmt::Debug,
// {
//     type MipsOutput;

//     fn into_mips_node(self, translator: &mut MypsTranslator) -> Self::MipsOutput;
// }
