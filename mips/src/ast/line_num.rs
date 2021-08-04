use std::{fmt, fmt::Display};

use crate::ast::{MipsNode, Num, RegBase};
use crate::{Aliases, MipsError, MipsParser, MipsResult, Pair, Rule};
use ast_traits::{AstNode, IntoAst};

#[derive(Clone, Debug)]
pub struct LineAbs(pub Num);

impl LineAbs {
    pub fn reduce(self, aliases: &Aliases) -> MipsResult<Self> {
        Ok(Self(self.0.reduce(aliases)?))
    }
}

impl<'i> MipsNode<'i> for LineAbs {
    fn as_reg_base(&self) -> Option<RegBase> {
        self.0.as_reg_base()
    }

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> {
        self.0.as_reg_base_mut()
    }

    fn as_alias(&self) -> Option<&String> {
        // self.0.as_alias()
        None
    }
}

impl From<f64> for LineAbs {
    fn from(n: f64) -> Self {
        Self(n.into())
    }
}

impl From<i64> for LineAbs {
    fn from(n: i64) -> Self {
        (n as f64).into()
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
pub struct LineRel(pub Num);

impl LineRel {
    pub fn reduce(self, aliases: &Aliases) -> MipsResult<Self> {
        Ok(Self(self.0.reduce(aliases)?))
    }
}

impl<'i> MipsNode<'i> for LineRel {
    fn as_reg_base(&self) -> Option<RegBase> {
        self.0.as_reg_base()
    }

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> {
        self.0.as_reg_base_mut()
    }

    fn as_alias(&self) -> Option<&String> {
        self.0.as_alias()
    }
}

impl From<f64> for LineRel {
    fn from(n: f64) -> Self {
        Self(n.into())
    }
}

impl From<i64> for LineRel {
    fn from(n: i64) -> Self {
        (n as f64).into()
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
