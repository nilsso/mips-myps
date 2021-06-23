use std::cell::RefCell;
use std::rc::Rc;
use std::{fmt, fmt::Display};

use pest::iterators::Pair;

use crate::ast_traits::OnlyInner;
use crate::mips::ast::{MipsNode, Var};
use crate::mips::{Alias, Mips, MipsError, MipsParser, MipsResult, Rule};

#[derive(Clone, Debug)]
pub struct RegRaw {
    pub(crate) unique_id: usize,
    pub(crate) index: usize,
    pub(crate) fixed: bool,
    pub(crate) lifetime: (usize, usize),
}

impl Display for RegRaw {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "r{}", self.index)
    }
}

#[derive(Clone, Debug)]
pub struct Reg(pub(crate) Rc<RefCell<RegRaw>>);

impl Reg {
    pub fn new(unique_id: usize, index: usize, fixed: bool, start: usize) -> Self {
        let lifetime = (start, start);
        let raw_reg = RegRaw { unique_id, index, fixed, lifetime };
        let raw = Rc::new(RefCell::new(raw_reg));
        Self(raw)
    }

    pub fn unique_id(&self) -> usize {
        self.0.borrow().unique_id
    }

    pub fn index(&self) -> usize {
        self.0.borrow().index
    }

    pub fn fixed(&self) -> bool {
        self.0.borrow().fixed
    }

    pub fn lifetime(&self) -> (usize, usize) {
        self.0.borrow().lifetime
    }

    pub fn update_lifetime(&self, s_opt: Option<usize>, e_opt: Option<usize>) {
        let (s, e) = &mut self.0.borrow_mut().lifetime;
        if let Some(new_s) = s_opt {
            *s = new_s;
        }
        if let Some(new_e) = e_opt {
            *e = new_e;
        }
    }
}

impl<'i> MipsNode<'i, Rule, MipsParser> for Reg {
    type Output = Var;

    const RULE: Rule = Rule::var;

    fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> MipsResult<Self::Output> {
        // println!("try reg from {:?}", pair);
        fn helper(mips: &mut Mips, pair: Pair<Rule>) -> MipsResult<(String, Var)> {
            match pair.as_rule() {
                Rule::var => helper(mips, pair.only_inner()?),
                Rule::reg => {
                    let s = pair.as_str();
                    let indirections = s.bytes().filter(|b| *b == b'r').count() - 1;
                    let index = pair.only_inner()?.as_str().parse()?;
                    let name = s.to_owned();
                    let reg = mips.new_reg(index, true);
                    let var = Var::Reg { indirections, reg };

                    Ok((name, var))
                }
                Rule::alias => {
                    let s = pair.as_str();
                    let name = s.to_owned();
                    let reg = mips.new_reg(mips.get_reg(s)?.index(), false);
                    let var = Var::Alias {
                        name: name.clone(),
                        reg,
                    };

                    Ok((name, var))
                }
                _ => {
                    Err(MipsError::arg_wrong_kind("a register", pair.as_str(), pair.as_rule()))
                }
            }
        }

        let (name, var) = helper(mips, pair)?;
        mips.aliases.insert(name, Alias::Var(var.clone()));
        Ok(var)
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.borrow())
    }
}
