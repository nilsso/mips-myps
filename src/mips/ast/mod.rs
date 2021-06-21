mod reg;
pub use reg::{RegBase, Reg};

mod num;
pub use num::Num;

mod arg;
pub use arg::Arg;

mod unit;
pub use unit::Unit;

use std::fmt::Debug;

use pest::{RuleType, Parser};
use pest::iterators::Pair;

use crate::{Mips, MipsParser, Rule};

pub type AstError = ();
pub type AstResult<T> = Result<T, AstError>;

pub trait AstNode<'i, R, P, E>
where
    Self: Sized,
    R: RuleType,
    P: Parser<R>,
    E: Debug,
{
    type Output;

    const RULE: R;

    fn try_from_pair(mips: &mut Mips, pair: Pair<R>) -> Result<Self::Output, E>;
}

pub trait IntoAst<'i, R, P, E>
where
    Self: Sized,
    R: RuleType,
    P: Parser<R>,
    E: Debug,
{
    fn try_into_ast<A: AstNode<'i, R, P, E, Output = A>>(self, mips: &mut Mips) -> Result<A, E>;

    fn into_ast<A: AstNode<'i, R, P, E, Output = A>>(self, mips: &mut Mips) -> A {
        Self::try_into_ast(self, mips).unwrap()
    }
}

impl<'i, R, P, E> IntoAst<'i, R, P, E> for Pair<'i, R>
where
    R: RuleType,
    P: Parser<R>,
    E: Debug,
{
    fn try_into_ast<A: AstNode<'i, R, P, E, Output = A>>(self, mips: &mut Mips) -> Result<A, E> {
        A::try_from_pair(mips, self)
    }
}

impl<'i> AstNode<'i, Rule, MipsParser, AstError> for String {
    type Output = Self;

    const RULE: Rule = Rule::num;

    fn try_from_pair(_mips: &mut Mips, pair: Pair<Rule>) -> AstResult<Self::Output> {
        // TODO: Lookup aliast from string
        Ok(pair.as_str().to_owned())
    }
}
