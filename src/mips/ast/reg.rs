use std::cell::RefCell;
use std::rc::Rc;
use std::{fmt, fmt::Display};

use pest::iterators::Pair;

use crate::ast_traits::OnlyInner;
use crate::mips::ast::{AstError, AstNode, AstResult};
use crate::mips::{Alias, Mips, MipsParser, Rule};

#[derive(Clone, Debug)]
pub struct RawRegBase {
    pub(crate) index: usize,
    pub(crate) lifetime: (usize, usize),
}

impl Display for RawRegBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "r{}", self.index)
    }
}

#[derive(Clone, Debug)]
pub struct RegBase {
    pub(crate) raw: Rc<RefCell<RawRegBase>>,
    pub(crate) indirections: usize,
}

impl RegBase {
    pub fn new(index: usize, indirections: usize, start: usize) -> Self {
        let lifetime = (start, start);
        let raw_reg = RawRegBase { index, lifetime };
        let raw = Rc::new(RefCell::new(raw_reg));
        Self { raw, indirections }
    }

    pub fn index(&self) -> usize {
        self.raw.borrow().index
    }

    pub fn lifetime(&self) -> (usize, usize) {
        self.raw.borrow().lifetime
    }

    pub fn update_lifetime(&mut self, s_opt: Option<usize>, e_opt: Option<usize>) {
        let (s, e) = &mut self.raw.borrow_mut().lifetime;
        if let Some(new_s) = s_opt {
            *s = new_s;
        }
        if let Some(new_e) = e_opt {
            *e = new_e;
        }
    }
}

impl Display for RegBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..self.indirections {
            write!(f, "r")?;
        }
        write!(f, "{}", self.raw.borrow())
    }
}

#[derive(Clone, Debug)]
pub enum Reg {
    Base(RegBase),
    Alias(String, Box<Reg>),
}

impl Reg {
    pub fn update_lifetime(&mut self, s_opt: Option<usize>, e_opt: Option<usize>) {
        match self {
            Self::Base(r) => r.update_lifetime(s_opt, e_opt),
            Self::Alias(_, r) => r.update_lifetime(s_opt, e_opt),
        }
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, AstError> for Reg {
    type Output = Self;

    const RULE: Rule = Rule::reg;

    fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> AstResult<Self::Output> {
        match pair.as_rule() {
            Rule::reg => {
                let indirections = pair.as_str().bytes().filter(|b| *b == b'r').count() - 1;
                let index = pair
                    .only_inner()
                    .unwrap()
                    .as_str()
                    .parse::<usize>()
                    .unwrap();
                let mut reg = mips.indexed_reg(index);
                reg.indirections = indirections;
                Ok(Self::Base(reg))
            }
            Rule::alias => {
                let alias = mips.get_alias(pair.as_str());
                let reg = match alias {
                    Some(Alias::Reg(reg)) => reg.clone(),
                    _ => panic!("{:?}", alias),
                };
                Ok(Self::Alias(pair.as_str().into(), Box::new(reg)))
            }
            _ => panic!("{:?}", pair),
        }
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Base(r) => write!(f, "{}", r),
            Self::Alias(s, _) => write!(f, "{}", s),
        }
    }
}
