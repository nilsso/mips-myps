#![feature(stmt_expr_attributes)]
#![feature(bool_to_option)]
#![feature(map_try_insert)]
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::IntoIterator;
use std::path::PathBuf;

use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};

use ast_traits::{AstNode, AstPairs, AstRule, IntoAst};
// use ast_common::AstCommonRule;

pub mod graph;
use graph::Graph;
// use crate::mips::ast::IntoMipsNode;

#[derive(Parser, Clone, Debug)]
#[grammar = "grammar.pest"]
pub struct MipsParser;

impl AstRule for Rule {
    fn eoi() -> Self {
        Self::EOI
    }
}

pub type Pair<'i> = pest::iterators::Pair<'i, Rule>;
pub type Pairs<'i> = pest::iterators::Pairs<'i, Rule>;

mod error;
pub use error::MipsError;

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for String {
    type Output = String;

    const RULE: Rule = Rule::alias;

    fn try_from_pair(pair: Pair) -> MipsResult<String> {
        Ok(pair.as_str().to_owned())
    }
}

pub mod ast;
use ast::{Arg, Dev, DevBase, Line, LineAbs, LineRel, MipsNode, Num, Reg, RegBase, Stmt};

pub type MipsResult<T> = Result<T, MipsError>;

#[derive(Clone, Debug)]
pub enum Alias {
    Dev(DevBase),
    Reg(RegBase),
    Num(f64),
}

impl From<DevBase> for Alias {
    fn from(dev_base: DevBase) -> Self {
        Self::Dev(dev_base)
    }
}

impl From<RegBase> for Alias {
    fn from(reg_base: RegBase) -> Self {
        Self::Reg(reg_base)
    }
}

impl From<f64> for Alias {
    fn from(n: f64) -> Self {
        Self::Num(n)
    }
}

impl TryFrom<&Alias> for Num {
    type Error = MipsError;

    fn try_from(alias: &Alias) -> MipsResult<Self> {
        match alias {
            Alias::Num(n) => Ok(Num::Lit(*n)),
            Alias::Reg(reg_base) => Ok(Num::Reg(*reg_base)),
            Alias::Dev(..) => Err(MipsError::alias_wrong_kind("a number or register", alias)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Aliases {
    pub map: BTreeMap<String, Alias>,
}

impl Aliases {
    pub fn new() -> Self {
        let map = BTreeMap::new();
        Self { map }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        // self.map.contains_key(key) // TODO: Stop-gap lowercase, until a constants table is implemented
        self.map.contains_key(key) || self.map.contains_key(&key.to_lowercase())
    }

    pub fn insert(&mut self, key: String, alias: Alias) -> Option<Alias> {
        self.map.insert(key, alias)
    }

    pub fn get(&self, key: &str) -> Option<&Alias> {
        // self.map.get(key) // TODO: Stop-gap lowercase, until a constants table is implemented
        self.map
            .get(key)
            .or_else(|| self.map.get(&key.to_lowercase()))
    }

    pub fn try_get(&self, key: &str) -> MipsResult<&Alias> {
        self.get(key).ok_or(MipsError::alias_undefined(key))
    }

    pub fn try_get_dev_base(&self, key: &str) -> MipsResult<DevBase> {
        let alias = self.try_get(key).unwrap();
        match alias {
            Alias::Dev(dev_lit) => Ok(dev_lit.clone()),
            _ => Err(MipsError::alias_wrong_kind("a device", alias)),
        }
    }

    pub fn get_reg_base(&self, key: &str) -> MipsResult<Option<RegBase>> {
        match self.try_get(key).unwrap() {
            Alias::Reg(reg_lit) => Ok(Some(reg_lit.clone())),
            _ => Ok(None),
        }
    }

    pub fn try_get_reg_base(&self, key: &str) -> MipsResult<RegBase> {
        let alias = self.try_get(key).unwrap();
        match alias {
            Alias::Reg(reg) => Ok(reg.clone()),
            _ => Err(MipsError::alias_wrong_kind("a register", alias)),
        }
    }
}

// TODO: Probably going to want to have a separate table for constants
// TODO: REALLY need to add a constants map!
impl Default for Aliases {
    fn default() -> Self {
        let map = maplit::btreemap! {
            "db".into() => Alias::Dev(DevBase::DB),
            "sp".into() => Alias::Reg(RegBase::SP),
            "ra".into() => Alias::Reg(RegBase::RA),
            // Back read modes
            "average".into() => Alias::Num(0.0),
            "avg".into() => Alias::Num(0.0),
            "sum".into() => Alias::Num(1.0),
            "min".into() => Alias::Num(2.0),
            "max".into() => Alias::Num(3.0),
            // Reagent modes
            "contents".into() => Alias::Num(0.0),
            "required".into() => Alias::Num(1.0),
            "recipe".into() => Alias::Num(2.0),

            "horizontal".into() => Alias::Num(20.0),
            "vertical".into() => Alias::Num(21.0),
        };
        Self { map }
    }
}

#[derive(Clone, Debug)]
pub struct Mips {
    //     // pub registers: Vec<RegShared>,
    // pub labels: BTreeMap<String, usize>,
    pub aliases: Aliases,
    pub present_aliases: BTreeSet<String>,
    pub tags: BTreeSet<String>,
    pub lines: Vec<Line>,
    pub next_unique_id: usize,
    // pub scopes: Vec<Range<usize>>,
}

impl Default for Mips {
    fn default() -> Self {
        // let registers = Vec::new();
        // let labels = BTreeMap::new();
        let aliases = Aliases::default();
        let present_aliases = BTreeSet::new();
        let tags = BTreeSet::new();
        let lines = Vec::new();
        // let scopes = Vec::new();
        Self {
            // registers,
            // labels,
            aliases,
            present_aliases,
            tags,
            lines,
            // scopes,
            next_unique_id: 0,
        }
    }
}

// #[derive(Clone, Debug)]
// pub struct Lifetime {
//     index: usize,
//     s: usize,
//     e: usize,
// }

impl Mips {
    pub fn lex_file<P: Into<PathBuf> + std::fmt::Debug>(path: P) -> Result<Self, String> {
        let path_string = format!("{:?}", path);
        let f = File::open(path.into()).expect(&path_string);
        let f = BufReader::new(f);
        let lines = f.lines().collect::<Result<Vec<_>, _>>().unwrap();
        // lex_lines(lines.into_iter())
        let mut mips = Self::default();
        mips.parse_lines(lines).unwrap();
        Ok(mips)
    }

    // pub fn default_with_source(source: &str) -> Result<Self, String> {
    //     let mut mips = Self::default();
    //     mips.parse_source(source)?;
    //     // mips.scopes.push(0..mips.lines.len());
    //     mips.lex()?;
    //     Ok(mips)
    // }

    pub fn default_with_lines(lines: Vec<Line>) -> Result<Self, String> {
        let mut mips = Self::default();
        mips.lines.extend(lines);
        // mips.scopes = scopes;
        mips.lex().unwrap();
        Ok(mips)
    }

    fn parse_line(&mut self, line: &str) -> Result<(), String> {
        let pairs = MipsParser::parse(Rule::line, line)
            .map_err(|err| {
                format!(
                    "(MIPS error) Line parsing error\nLine: \"{}\"\nError: {:?}",
                    line, err
                )
            })
            .unwrap();

        let line_pair = pairs
            .only_pair()
            .map_err(|err| {
                format!(
                    "(MIPS error) Too many inner pairs\nLine: \"{}\"\nError: {:?}",
                    line, err
                )
            })
            .unwrap();

        let line = line_pair
            .try_into_ast::<Line>()
            .map_err(|err| {
                format!(
                    "(MIPS error) Instruction parsing error\nLine: \"{}\"\nError: {}",
                    line, err
                )
            })
            .unwrap();

        self.lines.push(line);
        Ok(())
    }

    fn parse_lines<I: IntoIterator<Item = String>>(&mut self, lines: I) -> Result<(), String> {
        for line in lines.into_iter() {
            self.parse_line(&line).unwrap();
        }
        Ok(())
    }

    // fn parse_source(&mut self, source: &str) -> Result<(), String> {
    //     self.parse_lines(source.trim_end().split("\n"))
    // }

    fn lex(&mut self) -> Result<(), String> {
        // Line tag pass
        for (i, line) in self.lines.iter().enumerate() {
            let Line { stmt, .. } = line;
            if let Stmt::Tag([Arg::String(tag)]) = stmt {
                self.aliases.insert(tag.clone(), Alias::Num(i as f64));
                self.present_aliases.insert(tag.clone());
                self.tags.insert(tag.clone());
            }
        }
        // Variable pass
        for i in 0..self.lines.len() {
            self.lex_line(i).unwrap();
            // self.lex_line(i)
            //     .map_err(|err| {
            //         let line = &self.lines[i];
            //         format!(
            //             "(MIPS lexer error) Line {}\nLine: {}\nError: {}",
            //             i, line, err
            //         )
            //     })
            //     .unwrap();
        }
        Ok(())
    }

    fn lex_line(&mut self, i: usize) -> MipsResult<()> {
        // Check that aliases are present
        for arg in self.lines[i].stmt.iter_args().skip(1) {
            match arg {
                // Arg::Dev,
                Arg::Reg(Reg::Alias { key, .. })
                | Arg::Num(Num::Alias(key))
                | Arg::LineAbs(LineAbs(Num::Alias(key)))
                | Arg::LineRel(LineRel(Num::Alias(key))) => {
                    if !self.aliases.contains_key(key) {
                        return Err(MipsError::alias_undefined(key));
                    }
                }
                _ => {}
            }
        }
        // Insert aliases and definitions
        let Line { stmt, comment_opt } = &mut self.lines[i];
        match stmt {
            Stmt::Alias([Arg::String(key), Arg::Reg(reg)]) => {
                let mut reg_base = match reg {
                    Reg::Base(reg_base) => reg_base.clone(),
                    Reg::Alias { key, .. } => self.aliases.try_get_reg_base(key).unwrap(),
                };
                if let Some(comment) = comment_opt {
                    if comment.contains("FIX") {
                        reg_base.set_fixed(true);
                    }
                }
                let alias = Alias::Reg(reg_base);
                self.aliases.insert(key.clone(), alias);
                self.present_aliases.insert(key.clone());
            }
            Stmt::Alias([Arg::String(key), Arg::Dev(dev)]) => {
                let dev_base = match dev {
                    Dev::Base(dev_base) => dev_base.clone(),
                    Dev::Alias(key) => self.aliases.try_get_dev_base(key).unwrap(),
                };
                let alias = Alias::Dev(dev_base);
                self.aliases.insert(key.clone(), alias);
                self.present_aliases.insert(key.clone());
            }
            Stmt::Define([Arg::String(key), Arg::Num(Num::Lit(n))]) => {
                let alias = Alias::Num(*n);
                self.aliases.insert(key.clone(), alias);
                self.present_aliases.insert(key.clone());
            }
            _ => {
                if let Some(Arg::Reg(Reg::Base(mut reg_base))) = stmt.iter_args_mut().next() {
                    if let Some(comment) = comment_opt {
                        if comment.contains("FIX") {
                            reg_base.set_fixed(true);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn optimize(&self, conf: OptimizationConfig) -> Result<Mips, String> {
        let mut mips = self.clone();

        let tag_lines = mips
            .lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| {
                if let Stmt::Tag([key]) = &line.stmt {
                    let a = Some((key.to_string(), i));
                    a
                } else {
                    None
                }
            })
            .collect::<BTreeMap<String, usize>>();

        mips.lex().unwrap();

        if conf.optimize_registers {
            let lifetimes = mips.analyze_lifetimes();
            let n = lifetimes.len();
            let node_iter = lifetimes
                .iter()
                .map(|(index, _)| *index)
                .collect::<BTreeSet<usize>>()
                .into_iter()
                .map(|index| (index, index));
            let edge_iter = (0..n)
                .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
                .filter_map(|(i, j)| {
                    let (i_index, (i_s, i_e)) = &lifetimes[i];
                    let (j_index, (j_s, j_e)) = &lifetimes[j];
                    ((i_s < j_e) && (j_s < i_e)).then_some((*i_index, *j_index))
                });
            let graph = Graph::from_edges(node_iter.chain(edge_iter)).color();
            let colors = graph
                .into_nodes()
                .map(|node| {
                    let index = node.index();
                    let color = node.color().unwrap();
                    (index, color)
                })
                .collect::<BTreeMap<_, _>>();
            for line in mips.lines.iter_mut() {
                for arg in line.stmt.iter_args_mut() {
                    if let Arg::Dev(Dev::Base(DevBase::Lit(dev_lit))) = arg {
                        if dev_lit.indirections > 0 {
                            if let Some(new_index) = colors.get(&dev_lit.index) {
                                dev_lit.index = *new_index;
                            }
                        }
                    } else if let Some(RegBase::Lit(reg_lit)) = arg.as_reg_base_mut() {
                        if let Some(new_index) = colors.get(&reg_lit.index) {
                            reg_lit.index = *new_index;
                        }
                    }
                }
            }
        }

        // Define/alias/tag replacement and comment removal pass
        for (_i, line) in mips.lines.iter_mut().enumerate() {
            if conf.remove_comments {
                line.comment_opt = None;
            }
            match line {
                // Replace defines
                Line {
                    stmt: Stmt::Define(..),
                    comment_opt,
                } if conf.remove_defines => {
                    let comment_opt = comment_opt.take();
                    *line = Line {
                        stmt: Stmt::Empty([]),
                        comment_opt,
                    };
                }
                // Replace aliases
                Line {
                    stmt: Stmt::Alias([_, rhs], ..),
                    comment_opt,
                } if matches!(rhs, Arg::Dev(..)) && conf.remove_dev_aliases
                    || matches!(rhs, Arg::Reg(..)) && conf.remove_reg_aliases =>
                {
                    let comment_opt = comment_opt.take();
                    *line = Line {
                        stmt: Stmt::Empty([]),
                        comment_opt,
                    };
                }
                // Replace tags
                Line {
                    stmt: Stmt::Tag(_),
                    comment_opt,
                } if conf.remove_tags => {
                    let comment_opt = comment_opt.take();
                    *line = Line {
                        stmt: Stmt::Empty([]),
                        comment_opt,
                    };
                }
                Line { .. } => {}
            }
        }
        // Argument and comment replacement pass
        for line in mips.lines.iter_mut() {
            for arg in line.stmt.iter_args_mut() {
                match arg {
                    Arg::LineAbs(LineAbs(num)) if conf.remove_tags => {
                        if let Some(key) = num.as_alias() {
                            let i = tag_lines.get(key).expect(key);
                            *num = Num::Lit(*i as f64);
                        }
                    }
                    Arg::Num(num) => {
                        if let Some(key) = num.as_alias() {
                            if let Some(alias) = mips.aliases.get(key) {
                                match alias {
                                    Alias::Num(n) => {
                                        if conf.remove_defines {
                                            *arg = Arg::Num(Num::Lit(*n));
                                        }
                                    }
                                    Alias::Reg(reg_base) => {
                                        if conf.remove_reg_aliases {
                                            *arg = Arg::Reg(Reg::Base(*reg_base));
                                        }
                                    }
                                    Alias::Dev(_) => {
                                        panic!();
                                    }
                                }
                            }
                            // *num = Num::Lit()
                        }
                    }
                    Arg::Reg(Reg::Alias { key, .. }) => {
                        if conf.remove_reg_aliases {
                            let reg_base = mips.aliases.try_get_reg_base(key).unwrap();
                            *arg = Arg::Reg(Reg::Base(reg_base));
                        }
                    }
                    Arg::Dev(Dev::Alias(key)) => {
                        if conf.remove_dev_aliases {
                            let dev_base = mips.aliases.try_get_dev_base(key).unwrap();
                            *arg = Arg::Dev(Dev::Base(dev_base));
                        }
                    }
                    // Arg::String(key) => {
                    // }
                    _ => {}
                }
            }
        }
        // Remove empty lines
        if conf.remove_empty || conf.remove_empty_comments {
            let mut i = 0;
            while i < mips.lines.len() {
                let Line { stmt, comment_opt } = &mut mips.lines[i];
                match stmt {
                    Stmt::Empty(..)
                        if (conf.remove_empty
                            && (comment_opt.is_none() || conf.remove_empty_comments)) =>
                    {
                        mips.remove_line(i);
                    }
                    _ => i += 1,
                }
            }
        }
        // // Remove extraneous instructions
        // let mut i = 0;
        // while i < mips.lines.len() {
        //     let Line { stmt, comment_opt } = &mut mips.lines[i];
        //     match stmt {
        //         Stmt::Empty(..)
        //             if (conf.remove_empty
        //                 && (conf.remove_empty_comments || comment_opt.is_none())) =>
        //         {
        //             mips.remove_line(i);
        //         }
        //         Stmt::Define([Arg::String(key), ..], ..) if conf.remove_defines => {
        //             mips.present_aliases.remove(key);
        //             mips.remove_line(i);
        //         }
        //         Stmt::Alias([Arg::String(key), rhs], ..)
        //             if matches!(rhs, Arg::Dev(..)) && conf.remove_dev_aliases
        //                 || matches!(rhs, Arg::Reg(..)) && conf.remove_reg_aliases =>
        //         {
        //             mips.present_aliases.remove(key);
        //             mips.remove_line(i);
        //         }
        //         Stmt::Tag([Arg::String(key)], ..) if conf.remove_tags => {
        //             let alias = Alias::Num(i as f64);
        //             mips.aliases.insert(key.clone(), alias);
        //             mips.present_aliases.remove(key);
        //             mips.remove_line(i);
        //         }
        //         _ => i += 1,
        //     }
        // }

        // // Re-lex
        // mips.lex_lines()?;

        // // Reduce instructions
        // let mut lines = mips.lines.clone();
        // for line in lines.iter_mut() {
        //     line.stmt.reduce_args(&mips.aliases).ok();
        // }
        // mips.lines = lines;

        Ok(mips)
    }

    /// Remove a line safely
    ///
    /// Removes a line safely by keeping in mind edge cases:
    /// * adjusting relative branches that encompass the line to remove
    pub fn remove_line(&mut self, i: usize) {
        use crate::ast::FixMode;

        // Adjust relative branches encompassing the line to remove
        for (j, line) in self.lines.iter_mut().enumerate().filter(|&(j, _)| i != j) {
            for arg in line.stmt.iter_args_mut() {
                if let Some(reg_lit) = arg.as_reg_lit_mut() {
                    if let FixMode::Scoped(s, e) = &mut reg_lit.fix_mode {
                        if *s > i {
                            *s -= 1;
                        }
                        if *e > i {
                            *e -= 1;
                        }
                    }
                }
                if let Arg::LineAbs(LineAbs(Num::Lit(n))) = arg {
                    if (i as f64) < *n {
                        *n -= 1_f64;
                    }
                }
                if let Arg::LineRel(LineRel(Num::Lit(n))) = arg {
                    use std::cmp::Ordering::*;
                    let i = i as isize;
                    let j = j as isize;
                    let k = *n as isize;
                    match j.cmp(&i) {
                        Less => {
                            if (j..(j + k)).contains(&i) {
                                *n -= 1_f64;
                            }
                        }
                        Greater => {
                            if ((j + k)..j).contains(&i) {
                                *n += 1_f64;
                            }
                        }
                        Equal => {
                            // TODO: If line removed is also a relative jump, then maybe
                            // redirect any relative jumps to *this* line to where *this* jump
                            // is pointing
                            unreachable!();
                        }
                    }
                }
            }
        }
        // Adjust scope ends sitting beyond the line to remove
        // for scope in self.scopes.iter_mut() {
        //     if scope.start > i {
        //         scope.start -= 1;
        //     }
        //     if scope.end > i {
        //         scope.end -= 1;
        //     }
        // }
        self.lines.remove(i);
    }

    pub fn analyze_lifetimes(&self) -> Vec<(usize, (usize, usize))> {
        use crate::ast::{FixMode, RegLit};

        let mut res = Vec::new();
        let mut lifetimes = BTreeMap::<usize, (usize, usize)>::new();

        // for scope in scopes.iter() {
        //     let lifetime = Lifetime {
        //         index: *index,
        //         s: range.start,
        //         e: range.end,
        //     };
        //     res.push(lifetime);
        // }

        let scope_tuple = |i, fix_mode| {
            match fix_mode {
                FixMode::None => (i, i),
                FixMode::Fixed => (0, self.lines.len() - 1),
                FixMode::Scoped(s, e) => (s, e),
            }
        };

        // Fixed/scoped pass
        for (line_num, line) in self.lines.iter().enumerate() {
            for (_arg, reg_lit) in line.stmt.args().iter().rev().filter_map(|arg| {
                // arg.as_reg_lit()
                let opt = arg.get_reg_lit(self);
                opt.expect(&format!(
                    "Fatal error analyzing lifetimes\nLINE {}: {:?}\nARG: {:?}",
                    line_num, line, arg
                ))
                .map(|reg_lit| (arg, reg_lit))
            }) {
                #[rustfmt::skip]
                let RegLit { index, fix_mode, .. } = reg_lit;
                match fix_mode {
                    FixMode::None => {}
                    FixMode::Fixed | FixMode::Scoped(..) => {
                        let (s, e) = scope_tuple(line_num, fix_mode);
                        if let Some((s_old, e_old)) = lifetimes.get_mut(&index) {
                            *s_old = s.min(*s_old);
                            *e_old = e.max(*e_old);
                        } else {
                            lifetimes.insert(index, (s, e));
                        }
                    }
                }
            }
        }

        // Normal pass (as l-values and r-values)
        for (line_num, line) in self.lines.iter().enumerate()
        // NOTE: What was this filter for?
        // .filter(|(_, line)| !matches!(line.stmt, Stmt::Alias(..)))
        {
            for (arg, reg_lit) in line.stmt.args().iter().rev().filter_map(|arg| {
                // arg.as_reg_lit()
                arg.get_reg_lit(self)
                    .expect(&format!(
                        "Fatal error analyzing lifetimes\nLINE {}: {:?}\nARG: {:?}",
                        line_num, line, arg
                    ))
                    .map(|reg_lit| (arg, reg_lit))
            }) {
                #[rustfmt::skip]
                let RegLit { index, fix_mode, .. } = reg_lit;
                if matches!(arg, Arg::Reg(..)) {
                    // As l-value
                    match fix_mode {
                        FixMode::Fixed | FixMode::Scoped(..) => {}
                        FixMode::None => {
                            let new_scope = scope_tuple(line_num, fix_mode);
                            let (s_new, _) = new_scope;
                            if let Some((s_old, e_old)) = lifetimes.get_mut(&index) {
                                if s_new >= *e_old {
                                    res.push((index, (*s_old, *e_old)));
                                    lifetimes.insert(index, new_scope);
                                }
                            } else {
                                lifetimes.insert(index, new_scope);
                            }
                        }
                    }
                } else {
                    // As r-value
                    let (_s, e) = lifetimes.entry(index).or_insert((line_num, line_num));
                    if *e < line_num {
                        *e = line_num;
                    }
                }
            }
        }
        res.extend(lifetimes);
        res
    }

    pub fn interference_graph(&self) -> String {
        let lifetimes = self.analyze_lifetimes();

        let mut output = "LIFETIMES:\n         ".to_owned();
        let n = self.lines.len();
        for i in 0..n {
            if i % 10 == 0 {
                output.push_str(&format!("{:>2}", i));
            } else {
                output.push_str("  ");
            }
        }
        output.push_str("\n");
        for (i, (index, (s, e))) in lifetimes.into_iter().enumerate() {
            let index_s = format!("r{}", index);
            output.push_str(&format!("{:>3} {:>3} :", i, index_s));
            //         let (s, e) = reg.lifetime();
            for i in 0..n {
                #[rustfmt::skip]
                match (i == s || i == e, s <= i && i <= e) {
                    (false, true) => output.push_str(" -"),
                    (true,  true) => output.push_str(&format!("{:>2}", i % 10)),
                    _                     => output.push_str(" |"),
                }
            }
            output.push_str("\n");
        }
        output
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub remove_comments: bool,
    pub remove_empty: bool,
    pub remove_empty_comments: bool,
    pub remove_reg_aliases: bool,
    pub remove_dev_aliases: bool,
    pub remove_defines: bool,
    pub remove_tags: bool,
    pub optimize_registers: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            remove_comments: true,
            remove_empty: true,
            remove_empty_comments: true,
            remove_reg_aliases: true,
            remove_dev_aliases: true,
            remove_defines: true,
            remove_tags: true,
            optimize_registers: true,
        }
    }
}
