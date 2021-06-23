mod reg;
pub use reg::Reg;

mod var;
pub use var::Var;

mod num;
pub use num::Num;

mod arg;
pub use arg::Arg;

mod unit;
pub use unit::Unit;

use pest::iterators::Pair;
use pest::{Parser, RuleType};

use crate::mips::{Mips, MipsParser, Rule, MipsResult};

pub trait MipsNode<'i, R, P>
where
    Self: Sized,
    R: RuleType,
    P: Parser<R>,
{
    type Output;

    const RULE: R;

    fn try_from_pair(mips: &mut Mips, pair: Pair<R>) -> MipsResult<Self::Output>;
}

pub trait IntoMipsNode<'i, R, P>
where
    Self: Sized,
    R: RuleType,
    P: Parser<R>,
{
    fn try_into_ast<A: MipsNode<'i, R, P, Output = A>>(self, mips: &mut Mips) -> MipsResult<A>;

    fn into_ast<A: MipsNode<'i, R, P, Output = A>>(self, mips: &mut Mips) -> A {
        Self::try_into_ast(self, mips).unwrap()
    }
}

impl<'i, R, P> IntoMipsNode<'i, R, P> for Pair<'i, R>
where
    R: RuleType,
    P: Parser<R>,
{
    fn try_into_ast<A: MipsNode<'i, R, P, Output = A>>(self, mips: &mut Mips) -> MipsResult<A> {
        A::try_from_pair(mips, self)
    }
}

impl<'i> MipsNode<'i, Rule, MipsParser> for String {
    type Output = Self;

    const RULE: Rule = Rule::num;

    fn try_from_pair(_mips: &mut Mips, pair: Pair<Rule>) -> MipsResult<Self::Output> {
        // TODO: Lookup aliast from string
        Ok(pair.as_str().to_owned())
    }
}
