use std::{fmt, fmt::Display};

use crate::ast::MipsNode;
use crate::{Aliases, MipsError, MipsParser, MipsResult, Pair, Rule};
use ast_traits::{AstNode, AstPair, IntoAst};

#[derive(Copy, Clone, Hash, Debug)]
pub enum FixMode {
    None,
    Fixed,
    Scoped(usize, usize),
}

impl From<bool> for FixMode {
    fn from(fixed: bool) -> Self {
        if fixed {
            Self::Fixed
        } else {
            Self::None
        }
    }
}

impl From<&FixMode> for bool {
    fn from(fix_mode: &FixMode) -> bool {
        match fix_mode {
            FixMode::Fixed | FixMode::Scoped(..) => true,
            FixMode::None => false,
        }
    }
}

#[derive(Copy, Clone, Hash, Debug)]
pub struct RegLit {
    pub index: usize,
    pub indirections: usize,
    pub fix_mode: FixMode,
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for RegLit {
    type Output = Self;

    const RULE: Rule = Rule::reg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::reg => {
                let indirections = pair.as_str().bytes().filter(|b| *b == b'r').count() - 1;
                let index = pair.only_inner()?.as_str().parse()?;
                Ok(Self {
                    index,
                    indirections,
                    fix_mode: FixMode::None,
                })
            }
            _ => Err(MipsError::pair_wrong_rule("a literal register", pair)),
        }
    }
}

impl Display for RegLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..(self.indirections + 1) {
            write!(f, "r")?;
        }
        write!(f, "{}", self.index)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RegBase {
    SP,
    RA,
    Lit(RegLit),
}

impl RegBase {
    pub fn new_lit(index: usize, indirections: usize, fixed: bool) -> Self {
        let fix_mode = fixed.into();
        Self::Lit(RegLit {
            index,
            indirections,
            fix_mode,
        })
    }

    pub fn index(&self) -> usize {
        match self {
            Self::Lit(RegLit { index, .. }) => *index,
            Self::SP => 16,
            Self::RA => 17,
        }
    }

    pub fn fixed(&self) -> bool {
        match self {
            Self::Lit(RegLit { fix_mode, .. }) => fix_mode.into(),
            _ => false,
        }
    }

    pub fn set_fixed(&mut self, new_fixed: bool) {
        if let Self::Lit(RegLit { fix_mode, .. }) = self {
            *fix_mode = new_fixed.into();
        }
    }

    pub fn as_reg_lit(&self) -> Option<&RegLit> {
        match self {
            Self::Lit(reg_lit) => Some(reg_lit),
            _ => None,
        }
    }

    pub fn as_reg_lit_mut(&mut self) -> Option<&mut RegLit> {
        match self {
            Self::Lit(reg_lit) => Some(reg_lit),
            _ => None,
        }
    }
}

impl From<RegLit> for RegBase {
    fn from(reg_lit: RegLit) -> Self {
        Self::Lit(reg_lit)
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for RegBase {
    type Output = Self;

    const RULE: Rule = Rule::reg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::reg => {
                let s = pair.as_str();
                match s {
                    "sp" => Ok(Self::SP),
                    "ra" => Ok(Self::RA),
                    _ => {
                        let reg_lit = pair.try_into_ast()?;
                        Ok(Self::Lit(reg_lit))
                    }
                }
            }
            _ => Err(MipsError::pair_wrong_rule("a base register", pair)),
        }
    }
}

impl Display for RegBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SP => write!(f, "sp"),
            Self::RA => write!(f, "ra"),
            Self::Lit(reg_lit) => write!(f, "{}", reg_lit),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Reg {
    Base(RegBase),
    Alias { key: String, fixed: bool },
}

impl Reg {
    pub fn fixed(&self) -> bool {
        match self {
            Self::Base(reg_base) => reg_base.fixed(),
            _ => false,
        }
    }

    pub fn set_fixed(&mut self, new_fixed: bool) {
        if let Self::Base(reg_base) = self {
            reg_base.set_fixed(new_fixed);
        }
    }

    pub fn as_reg_lit(&self) -> Option<&RegLit> {
        match self {
            Self::Base(reg_base) => reg_base.as_reg_lit(),
            _ => None,
        }
    }

    pub fn as_reg_lit_mut(&mut self) -> Option<&mut RegLit> {
        match self {
            Self::Base(reg_base) => reg_base.as_reg_lit_mut(),
            _ => None,
        }
    }

    pub fn reduce(self, aliases: &Aliases) -> MipsResult<Self> {
        match self {
            Self::Alias { key, .. } => Ok(Self::Base(aliases.try_get_reg_base(&key)?)),
            _ => Ok(self),
        }
    }
}

impl<'i> MipsNode<'i> for Reg {
    fn as_reg_base(&self) -> Option<RegBase> {
        match self {
            Self::Base(reg_base) => Some(reg_base.clone()),
            _ => None,
        }
    }

    fn as_reg_base_mut(&mut self) -> Option<&mut RegBase> {
        match self {
            Self::Base(reg_base) => Some(reg_base),
            _ => None,
        }
    }

    fn as_alias(&self) -> Option<&String> {
        match self {
            Self::Alias { key, .. } => Some(key),
            _ => None,
        }
    }
}

impl From<RegBase> for Reg {
    fn from(reg_base: RegBase) -> Self {
        Self::Base(reg_base)
    }
}

impl From<RegLit> for Reg {
    fn from(reg_lit: RegLit) -> Self {
        Self::Base(reg_lit.into())
    }
}

impl From<String> for Reg {
    fn from(key: String) -> Self {
        Self::Alias { key, fixed: false }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for Reg {
    type Output = Self;

    const RULE: Rule = Rule::arg;

    fn try_from_pair(pair: Pair) -> MipsResult<Self::Output> {
        match pair.as_rule() {
            Rule::reg => Ok(Self::Base(pair.try_into_ast()?)),
            Rule::alias => {
                let key = pair.as_str().to_owned();
                let fixed = false;
                Ok(Self::Alias { key, fixed })
            }
            _ => Err(MipsError::pair_wrong_rule("a register", pair)),
        }
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Base(t) => write!(f, "{}", t),
            Self::Alias { key, .. } => write!(f, "{}", key),
        }
    }
}
