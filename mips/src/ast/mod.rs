use crate::{Mips, MipsError, MipsParser, MipsResult, Rule};
pub use ast_traits::AstNode;

// pub use ast_common::ModeRepr;
// pub type BatchMode = ast_common::BatchMode;
// pub type ReagentMode = ast_common::ReagentMode;
// pub type ReagentMode<'a> = ast_common::ReagentMode<'a, Rule, MipsParser, MipsError>;

pub trait MipsNode<'i>: AstNode<'i, Rule, MipsParser, MipsError> + std::fmt::Debug {
    fn as_reg_base(&self) -> Option<RegBase>;

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase>;

    fn as_alias(&self) -> Option<&String>;

    // fn reduce(self, mips: &Mips) -> MipsResult<Self> where Self: Sized;

    fn get_reg_base(&self, mips: &Mips) -> MipsResult<Option<RegBase>> {
        if let Some(reg_base) = self.as_reg_base() {
            Ok(Some(reg_base))
        } else if let Some(key) = self.as_alias() {
            Ok(mips.aliases.get_reg_base(key)?)
        } else {
            Ok(None)
        }
    }

    fn get_reg_lit(&self, mips: &Mips) -> MipsResult<Option<RegLit>> {
        let res = self.get_reg_base(mips);
        if let Some(RegBase::Lit(reg_lit)) = res? {
            Ok(Some(reg_lit))
        } else {
            Ok(None)
        }
    }
}

// impl<'i> MipsNode<'i> for BatchMode {
//     fn as_reg_base(&self) -> Option<RegBase> { None }
//     fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> { None }
//     fn as_alias(&self) -> Option<&String> { None }
//     fn reduce(self, _mips: &Mips) -> MipsResult<Self> {
//         match self {
//             Self::Avg(..) => Ok(Self::Avg(ModeRepr::Int)),
//             Self::Sum(..) => Ok(Self::Sum(ModeRepr::Int)),
//             Self::Min(..) => Ok(Self::Min(ModeRepr::Int)),
//             Self::Max(..) => Ok(Self::Max(ModeRepr::Int)),
//         }
//     }
// }

// impl<'i> MipsNode<'i> for ReagentMode {
//     fn as_reg_base(&self) -> Option<RegBase> { None }
//     fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> { None }
//     fn as_alias(&self) -> Option<&String> { None }
//     fn reduce(self, _mips: &Mips) -> MipsResult<Self> {
//         #[rustfmt::skip]
//         match self {
//             Self::Contents(..) => Ok(Self::Contents(ModeRepr::Int)),
//             Self::Required(..) => Ok(Self::Required(ModeRepr::Int)),
//             Self::Recipe(..)   => Ok(Self::Recipe(ModeRepr::Int)),
//         }
//     }
// }

mod dev;
pub use dev::{Dev, DevBase, DevLit};

mod reg;
pub use reg::{FixMode, Reg, RegBase, RegLit};

mod num;
pub use num::{Num, NumLit};

// mod var;
// pub use var::Var;

mod line_num;
pub use line_num::{LineAbs, LineRel};

mod arg;
pub use arg::{Arg, DevOrReg};

// mod mode;
// pub use mode::{BatchMode, ReagentMode};

mod stmt;
pub use stmt::Stmt;

mod line;
pub use line::Line;
