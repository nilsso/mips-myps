#![allow(unused_imports)]
use std::{fmt, fmt::Display, fmt::Debug};
use std::fs;
use std::io::Error as IOError;
use std::path::Path;

use pest::error::Error as PegError;
use pest::{Parser, RuleType};
use pest::iterators::{Pair, Pairs};

#[derive(Clone, Debug)]
pub enum AstErrorBase {
    PairsNotEnough,
    PairsTooMany,
    PairsWrongNum(usize, usize),
}

impl Display for AstErrorBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PairsNotEnough => write!(f, "Not enough pairs"),
            Self::PairsTooMany => write!(f, "Too many pairs"),
            Self::PairsWrongNum(expected, found) => write!(f, "Expected {} pairs, found {}", expected, found),
        }
    }
}

pub trait AstError<R: RuleType>: From<AstErrorBase> + From<PegError<R>> + From<IOError> + Debug {
    fn pairs_wrong_num(e: usize, f: usize) -> Self where Self: Sized {
        AstErrorBase::PairsWrongNum(e, f).into()
    }
}

impl<R, E> AstError<R> for E
where
    R: RuleType,
    E: From<AstErrorBase> + From<PegError<R>> + From<IOError> + Debug,
{}

pub trait AstPairs<'i, R: RuleType> {
    fn next_pair(&mut self) -> Result<Pair<'i, R>, AstErrorBase>;
    fn final_pair(&mut self) -> Result<Pair<'i, R>, AstErrorBase>;
    fn only_pair(self) -> Result<Pair<'i, R>, AstErrorBase>;
}

impl<'i, R: RuleType> AstPairs<'i, R> for Pairs<'i, R> {
    fn next_pair(&mut self) -> Result<Pair<'i, R>, AstErrorBase> {
        self.next().ok_or(AstErrorBase::PairsNotEnough)
    }

    fn final_pair(&mut self) -> Result<Pair<'i, R>, AstErrorBase> {
        // let pair = self.next_pair()?;
        let pair = self.next_pair()?;
        if self.next().is_none() {
            Ok(pair)
        } else {
            Err(AstErrorBase::PairsTooMany)
        }
    }

    fn only_pair(mut self) -> Result<Pair<'i, R>, AstErrorBase> {
        self.final_pair()
    }
}

pub trait AstPair<'i, R: RuleType> {
    fn only_inner(self) -> Result<Pair<'i, R>, AstErrorBase>;
}

impl<'i, R: RuleType> AstPair<'i, R> for Pair<'i, R> {
    fn only_inner(self) -> Result<Pair<'i, R>, AstErrorBase> {
        self.into_inner().final_pair()
    }
}


/// Abstract syntax tree conversion traits.
///
/// Provided an implementation of [`try_from_pair`](`AstNode::try_from_pair`),
/// provides additional conversion functions from `&str` and `&Path` as well as
/// error-less but panicking versions.
pub trait AstNode<'i, R, P, E>
where
    Self: Sized,
    R: RuleType,
    P: Parser<R>,
    E: AstError<R>,
{
    type Output: Debug;

    const RULE: R;

    fn try_from_pair(pair: Pair<R>) -> Result<Self::Output, E>;

    fn try_from_str<S: AsRef<str>>(source: &S) -> Result<Self::Output, E> {
        let mut pairs = P::parse(Self::RULE, source.as_ref())?;
        let pair = pairs.final_pair()?;
        Self::try_from_pair(pair)
    }

    fn try_from_file(path: &Path) -> Result<Self::Output, E> {
        let input = fs::read_to_string(path)?;
        Self::try_from_str(&input)
    }

    fn from_pair(pair: Pair<R>) -> Self::Output {
        Self::try_from_pair(pair).unwrap()
    }

    fn from_str<S: AsRef<str>>(source: &S) -> Self::Output {
        Self::try_from_str(source).unwrap()
    }

    fn from_file(path: &Path) -> Self::Output {
        Self::try_from_file(path).unwrap()
    }
}

/// Pair into [`AstNode`] conversion trait.
pub trait IntoAst<'i, R, P, E>
where
    Self: Sized,
    R: RuleType,
    P: Parser<R>,
    E: AstError<R> + From<PegError<R>> + From<IOError> + Debug,
{
    fn try_into_ast<A: AstNode<'i, R, P, E, Output = A>>(self) -> Result<A, E>;

    fn into_ast<A: AstNode<'i, R, P, E, Output = A>>(self) -> A {
        Self::try_into_ast(self).unwrap()
    }
}

impl<'i, R, P, E> IntoAst<'i, R, P, E> for Pair<'i, R>
where
    R: RuleType,
    P: Parser<R>,
    E: AstError<R> + From<PegError<R>> + From<IOError> + Debug,
{
    fn try_into_ast<A: AstNode<'i, R, P, E, Output = A>>(self) -> Result<A, E> {
        A::try_from_pair(self)
    }
}
