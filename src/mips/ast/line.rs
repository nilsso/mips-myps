use std::{fmt, fmt::Display};

use crate::ast_traits::{AstNode, AstPair, IntoAst};
use crate::mips::ast::{Reg, RegBase, Num, MipsNode};
use crate::mips::{Mips, MipsError, MipsParser, MipsResult, Pair, Rule};

#[derive(Clone, Debug)]
pub struct LineAbs(pub(crate) Num);

impl MipsNode for LineAbs {
    fn as_reg_base(&self) -> Option<RegBase> {
        self.0.as_reg_base()
    }

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> {
        self.0.as_reg_base_mut()
    }

    fn as_alias(&self) -> Option<&String> {
        self.0.as_alias()
    }

    fn reduce(self, mips: &Mips) -> MipsResult<Self> {
        Ok(Self(self.0.reduce(mips)?))
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for LineAbs {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self> {
        let num = pair.try_into_ast();
        num.map(LineAbs)
    }
}

impl Display for LineAbs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct LineRel(pub(crate) Num);

impl MipsNode for LineRel {
    fn as_reg_base(&self) -> Option<RegBase> {
        self.0.as_reg_base()
    }

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> {
        self.0.as_reg_base_mut()
    }

    fn as_alias(&self) -> Option<&String> {
        self.0.as_alias()
    }

    fn reduce(self, mips: &Mips) -> MipsResult<Self> {
        Ok(Self(self.0.reduce(mips)?))
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for LineRel {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self> {
        let num = pair.try_into_ast();
        num.map(LineRel)
    }
}

impl Display for LineRel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
