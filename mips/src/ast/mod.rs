use crate::{Mips, MipsResult};

mod dev;
pub use dev::{DevLit, DevBase, Dev};

mod reg;
pub use reg::{RegLit, RegBase, Reg};

mod num;
pub use num::{Num, NumLit};

// mod var;
// pub use var::Var;

mod line;
pub use line::{LineAbs, LineRel};

mod arg;
pub use arg::{Arg, DevOrReg};

mod mode;
pub use mode::{BatchMode, ReagentMode};

mod unit;
pub use unit::Unit;

pub trait MipsNode
where
    Self: std::fmt::Debug,
{
    fn as_reg_base(&self) -> Option<RegBase>;

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase>;

    fn as_alias(&self) -> Option<&String>;

    fn reduce(self, mips: &Mips) -> MipsResult<Self> where Self: Sized;

    fn get_reg_base(&self, mips: &Mips) -> MipsResult<Option<RegBase>> {
        if let Some(reg_base) = self.as_reg_base() {
            Ok(Some(reg_base))
        } else if let Some(key) = self.as_alias() {
            Ok(mips.get_reg_base(key)?)
        } else {
            Ok(None)
        }
    }

    fn get_reg_lit(&self, mips: &Mips) -> MipsResult<Option<RegLit>> {
        if let Some(RegBase::Lit(reg_lit)) = self.get_reg_base(mips)? {
            Ok(Some(reg_lit))
        } else {
            Ok(None)
        }
    }
}
