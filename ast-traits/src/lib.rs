#![feature(bool_to_option)]
#![allow(unused_imports)]
use std::fs;
use std::io::Error as IOError;
use std::path::Path;
use std::{fmt, fmt::Debug, fmt::Display};

use pest::error::Error as PegError;
use pest::iterators::{Pair, Pairs};
use pest::{Parser, RuleType};

pub trait AstRule: RuleType {
    fn eoi() -> Self;
}

#[derive(Clone, Debug)]
pub enum AstErrorBase {
    PairsNotEnough,
    PairsTooMany,
    PairsWrongNum(String),
    PairWrongRule(String),
}

impl AstErrorBase {
    pub fn pairs_wrong_num(expected: usize, found: usize) -> Self {
        Self::PairsWrongNum(format!("Expected {} pairs, found {}", expected, found))
    }

    pub fn pair_wrong_rule<'i, R: AstRule>(expected: &'static str, found: Pair<'i, R>) -> Self {
        Self::PairWrongRule(format!("Expected {} pair, found {:?}", expected, found))
    }
}

impl Display for AstErrorBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PairsNotEnough => write!(f, "Not enough pairs"),
            Self::PairsTooMany => write!(f, "Too many pairs"),
            Self::PairsWrongNum(s) => write!(f, "{}", s),
            Self::PairWrongRule(s) => write!(f, "{}", s),
        }
    }
}

pub trait AstError<R: AstRule>:
    From<AstErrorBase> + From<PegError<R>> + From<IOError> + Debug
{
    fn pairs_wrong_num(e: usize, f: usize) -> Self {
        AstErrorBase::pairs_wrong_num(e, f).into()
    }

    fn pair_wrong_rule<'i>(expected: &'static str, found: Pair<'i, R>) -> Self {
        AstErrorBase::pair_wrong_rule(expected, found).into()
    }
}

impl<R, E> AstError<R> for E
where
    R: AstRule,
    E: From<AstErrorBase> + From<PegError<R>> + From<IOError> + Debug,
{
}

pub trait AstPairs<'i, R: AstRule> {
    fn next_pair(&mut self) -> Result<Pair<'i, R>, AstErrorBase>;
    fn next_rule(&mut self, rule: R, rule_str: &'static str) -> Result<Pair<'i, R>, AstErrorBase>;

    fn final_pair(&mut self) -> Result<Pair<'i, R>, AstErrorBase>;
    fn final_rule(&mut self, rule: R, rule_str: &'static str) -> Result<Pair<'i, R>, AstErrorBase>;

    fn only_pair(self) -> Result<Pair<'i, R>, AstErrorBase>;
    fn only_rule(self, rule: R, rule_str: &'static str) -> Result<Pair<'i, R>, AstErrorBase>;
}

impl<'i, R: AstRule> AstPairs<'i, R> for Pairs<'i, R> {
    fn next_pair(&mut self) -> Result<Pair<'i, R>, AstErrorBase> {
        self.next().ok_or(AstErrorBase::PairsNotEnough)
    }

    fn next_rule(&mut self, rule: R, rule_str: &'static str) -> Result<Pair<'i, R>, AstErrorBase> {
        let pair = self.next_pair()?;
        if pair.as_rule() == rule {
            Ok(pair)
        } else {
            Err(AstErrorBase::pair_wrong_rule(rule_str, pair))
        }
    }

    fn final_pair(&mut self) -> Result<Pair<'i, R>, AstErrorBase> {
        let pair = self.next_pair()?;
        self.next()
            .map(|pair| pair.is_eoi())
            .unwrap_or(true)
            .then_some(pair)
            .ok_or(AstErrorBase::PairsTooMany)
    }

    fn final_rule(&mut self, rule: R, rule_str: &'static str) -> Result<Pair<'i, R>, AstErrorBase> {
        let pair = self.final_pair()?;
        if pair.as_rule() == rule {
            Ok(pair)
        } else {
            Err(AstErrorBase::pair_wrong_rule(rule_str, pair))
        }
    }

    fn only_pair(mut self) -> Result<Pair<'i, R>, AstErrorBase> {
        self.final_pair()
    }

    fn only_rule(mut self, rule: R, rule_str: &'static str) -> Result<Pair<'i, R>, AstErrorBase> {
        self.final_rule(rule, rule_str)
    }
}

pub trait AstPair<'i, R: AstRule> {
    fn is_eoi(&self) -> bool;

    fn first_inner(self) -> Result<Pair<'i, R>, AstErrorBase>;

    fn only_inner(self) -> Result<Pair<'i, R>, AstErrorBase>;
    fn only_rule(self, rule: R, rule_str: &'static str) -> Result<Pair<'i, R>, AstErrorBase>;
}

impl<'i, R: AstRule> AstPair<'i, R> for Pair<'i, R> {
    fn is_eoi(&self) -> bool {
        self.as_rule() == R::eoi()
    }

    fn first_inner(self) -> Result<Pair<'i, R>, AstErrorBase> {
        self.into_inner().next_pair()
    }

    fn only_inner(self) -> Result<Pair<'i, R>, AstErrorBase> {
        self.into_inner().final_pair()
    }

    fn only_rule(self, rule: R, rule_str: &'static str) -> Result<Pair<'i, R>, AstErrorBase> {
        let pair = self.only_inner()?;
        if pair.as_rule() == rule {
            Ok(pair)
        } else {
            Err(AstErrorBase::pair_wrong_rule(rule_str, pair))
        }
    }
}

/// Abstract syntax tree conversion traits.
///
/// Provided an implementation of [`try_from_pair`](`AstNode::try_from_pair`),
/// provides additional conversion functions from `&str` and `&Path` as well as
/// error-less but panicking versions.
pub trait AstNode<'i, R: AstRule, P: Parser<R>, E: AstError<R>>
where
    Self: Sized,
{
    type Output;

    const RULE: R;

    fn try_from_pair(pair: Pair<'i, R>) -> Result<Self::Output, E>;

    // fn try_from_str<'s, S: AsRef<str>>(source: &'s S) -> Result<Self::Output, E> {
    //     let mut pairs = P::parse(Self::RULE, source.as_ref())?;
    //     let pair = pairs.final_pair()?;
    //     Self::try_from_pair(pair)
    // }

    // fn try_from_file(path: &'i Path) -> Result<Self::Output, E> {
    //     let input = fs::read_to_string(path)?;
    //     Self::try_from_str(&input)
    // }

    fn from_pair(pair: Pair<'i, R>) -> Self::Output {
        Self::try_from_pair(pair).unwrap()
    }

    // fn from_str<S: AsRef<str>>(source: &'i S) -> Self::Output {
    //     Self::try_from_str(source).unwrap()
    // }

    // fn from_file(path: &'i Path) -> Self::Output {
    //     Self::try_from_file(path).unwrap()
    // }
}

// Pair into [`AstNode`] conversion trait.
pub trait IntoAst<'i, R, P, E>
where
    Self: Sized,
    R: AstRule,
    P: Parser<R>,
    E: AstError<R>,
{
    fn try_into_ast<A: AstNode<'i, R, P, E, Output = A>>(self) -> Result<A, E>;

    fn into_ast<A: AstNode<'i, R, P, E, Output = A>>(self) -> A {
        Self::try_into_ast(self).unwrap()
    }
}

impl<'i, R, P, E> IntoAst<'i, R, P, E> for Pair<'i, R>
where
    R: AstRule,
    P: Parser<R>,
    E: AstError<R>,
{
    fn try_into_ast<A: AstNode<'i, R, P, E, Output = A>>(self) -> Result<A, E> {
        A::try_from_pair(self)
    }
}
