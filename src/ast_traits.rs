#![allow(unused_imports)]
use std::path::Path;
use std::fs;
use std::fmt::Debug;
use std::io::Error as IOError;

use pest::{RuleType, Parser};
use pest::iterators::{Pair, Pairs};
use pest::error::Error as PegError;

/// Abstract syntax tree conversion error type.
#[derive(Debug)]
pub enum AstError {
    NotEnoughPairs,
    TooManyPairs,
}

// ================================================================================================
/// Get next [`Pair`](`pest::iterators::Pair`) from [`Pairs`](`pest::iterators::Pairs`) trait.
///
/// Provides blanket implementations over [`Pairs`](`pest::iterators::Pairs`).
pub trait NextPair<'i, R>
where
    Self: Iterator<Item = Pair<'i, R>>,
    R: RuleType,
{
    fn next_pair(&mut self) -> Result<Pair<'i, R>, AstError> {
        self.next().ok_or(AstError::NotEnoughPairs.into())
    }
}

impl<'i, R: RuleType> NextPair<'i, R> for Pairs<'i, R> {}
impl<'i, R: RuleType> NextPair<'i, R> for std::vec::IntoIter<Pair<'i, R>> {}

// ================================================================================================
pub trait PairsDone<'i, R>
where
    Self: Iterator<Item = Pair<'i, R>>,
    R: RuleType,
{
    fn done(&mut self) -> Result<(), AstError> {
        if self.next().is_some() {
            Err(AstError::TooManyPairs)
        } else {
            Ok(())
        }
    }
}

impl<'i, R> PairsDone<'i, R> for Pairs<'i, R>
where
    R: RuleType,
{}

// ================================================================================================
pub trait FinalPair<'i, R>
where
    Self: NextPair<'i, R> + PairsDone<'i, R> + Iterator<Item = Pair<'i, R>>,
    R: RuleType,
{
    fn final_pair(&mut self) -> Result<Pair<'i, R>, AstError> {
        let pair = self.next_pair()?;
        self.done()?;
        Ok(pair)
    }
}

impl<'i, R> FinalPair<'i, R> for Pairs<'i, R>
where
    R: RuleType,
{}

// ================================================================================================
pub trait OnlyInner<'i, R>
where
    R: RuleType,
{
    fn only_inner(self) -> Result<Pair<'i, R>, AstError>;
}

impl<'i, R> OnlyInner<'i, R> for Pairs<'i, R>
where
    R: RuleType,
{
    fn only_inner(mut self) -> Result<Pair<'i, R>, AstError> {
        self.final_pair()
    }
}

impl<'i, R> OnlyInner<'i, R> for Pair<'i, R>
where
    R: RuleType,
{
    fn only_inner(self) -> Result<Pair<'i, R>, AstError> {
        self.into_inner().only_inner()
    }
}

// impl<'i, R> FirstLastInne

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
    E: From<PegError<R>> + From<AstError> + From<IOError> + Debug,
{
    type Output: Debug;

    const RULE: R;

    fn try_from_pair(pair: Pair<R>) -> Result<Self::Output, E>;

    fn try_from_str<S: AsRef<str>>(source: &S) -> Result<Self::Output, E> {
        let pairs = P::parse(Self::RULE, source.as_ref())?;
        pairs.only_inner().map_err(E::from).and_then(Self::try_from_pair)
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
    E: From<PegError<R>> + From<AstError> + From<IOError> + Debug,
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
    E: From<PegError<R>> + From<AstError> + From<IOError> + Debug,
{
    fn try_into_ast<A: AstNode<'i, R, P, E, Output = A>>(self) -> Result<A, E> {
        A::try_from_pair(self)
    }
}

