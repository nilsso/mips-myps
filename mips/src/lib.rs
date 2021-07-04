#![feature(stmt_expr_attributes)]
#![feature(map_into_keys_values)]
#![feature(bool_to_option)]
use std::collections::{BTreeMap, BTreeSet};

use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};

use ast_traits::{AstNode, AstPairs, IntoAst, AstRule};
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
use ast::{Arg, Dev, DevBase, LineAbs, LineRel, Line, MipsNode, Num, Reg, RegBase, Stmt};

pub type MipsResult<T> = Result<T, MipsError>;

#[derive(Clone, Debug)]
pub enum Alias {
    Num(f64),
    Dev(DevBase),
    Reg(RegBase),
}

#[derive(Clone, Debug)]
pub struct Mips {
    //     // pub registers: Vec<RegShared>,
    // pub labels: BTreeMap<String, usize>,
    pub aliases: BTreeMap<String, Alias>,
    pub present_aliases: BTreeSet<String>,
    pub lines: Vec<Line>,
    pub next_unique_id: usize,
}

impl Default for Mips {
    fn default() -> Self {
        // let registers = Vec::new();
        // let labels = BTreeMap::new();
        let aliases = BTreeMap::new();
        let present_aliases = BTreeSet::new();
        let lines = Vec::new();
        Self {
            // registers,
            // labels,
            aliases,
            present_aliases,
            lines,
            next_unique_id: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Lifetime {
    index: usize,
    s: usize,
    e: usize,
}

impl Mips {
    pub fn get_alias(&self, key: &str) -> Option<&Alias> {
        self.aliases.get(key)
    }

    pub fn try_alias(&self, key: &str) -> MipsResult<&Alias> {
        self.get_alias(key).ok_or(MipsError::alias_undefined(key))
    }

    pub fn get_only_dev_base(&self, key: &str) -> MipsResult<DevBase> {
        let alias = self.try_alias(key)?;
        match alias {
            Alias::Dev(dev_lit) => Ok(dev_lit.clone()),
            _ => Err(MipsError::alias_wrong_kind("a device", alias)),
        }
    }

    pub fn get_reg_base(&self, key: &str) -> MipsResult<Option<RegBase>> {
        match self.try_alias(key)? {
            Alias::Reg(reg_lit) => Ok(Some(reg_lit.clone())),
            _ => Ok(None),
        }
    }

    pub fn get_only_reg_base(&self, key: &str) -> MipsResult<RegBase> {
        let alias = self.try_alias(key)?;
        match alias {
            Alias::Reg(reg) => Ok(reg.clone()),
            _ => Err(MipsError::alias_wrong_kind("a register", alias)),
        }
    }

    pub fn new(source: &str) -> Result<Self, String> {
        let mut mips = Mips::default();
        mips.parse_source(source)?;
        mips.lex_lines()?;
        Ok(mips)
    }

    fn parse_source(&mut self, source: &str) -> Result<(), String> {
        for line in source.trim_end().split("\n") {
            let pairs = MipsParser::parse(Rule::line, line).map_err(|err| {
                format!("Line parsing error\nLine: \"{}\"\nError: {:?}", line, err)
            })?;

            let line_pair = pairs.only_pair().map_err(|err| {
                format!("Too many inner pairs\nLine: \"{}\"\nError: {:?}", line, err)
            })?;

            let line = line_pair.try_into_ast::<Line>().map_err(|err| {
                format!(
                    "Instruction parsing error\nLine: \"{}\"\nError: {}",
                    line, err
                )
            })?;

            self.lines.push(line);
        }
        Ok(())
    }

    fn lex_lines(&mut self) -> Result<(), String> {
        // Line tag pass
        for (i, line) in self.lines.iter().enumerate() {
            let Line { stmt, .. } = line;
            if let Stmt::Tag([Arg::String(tag)]) = stmt {
                self.aliases.insert(tag.clone(), Alias::Num(i as f64));
                self.present_aliases.insert(tag.clone());
            }
        }
        // Variable pass
        for i in 0..self.lines.len() {
            self.lex_line(i).map_err(|err| {
                let line = &self.lines[i];
                format!("Lexer error on line {}\nLine: {}\nError: {}", i, line, err)
            })?;
        }
        Ok(())
    }

    fn lex_line(&mut self, i: usize) -> MipsResult<()> {
        // Check that aliases are present
        for arg in self.lines[i].stmt.iter_args() {
            match arg {
                // Arg::Dev,
                Arg::Reg(Reg::Alias(key))
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
        match &self.lines[i].stmt {
            Stmt::Alias([Arg::String(key), Arg::Reg(reg)]) => {
                let reg_base = match reg {
                    Reg::Base(reg_base) => reg_base.clone(),
                    Reg::Alias(key) => self.get_only_reg_base(key)?,
                };
                let alias = Alias::Reg(reg_base);
                self.aliases.insert(key.clone(), alias);
                self.present_aliases.insert(key.clone());
            }
            Stmt::Alias([Arg::String(key), Arg::Dev(dev)]) => {
                let dev_base = match dev {
                    Dev::Base(dev_base) => dev_base.clone(),
                    Dev::Alias(key) => self.get_only_dev_base(key)?,
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
            _ => {}
        }
        Ok(())
    }

    pub fn optimize(&self, conf: OptimizationConfig) -> Result<Mips, String> {
        let mut mips = self.clone();

        // Remove comments
        if conf.remove_comments {
            for line in mips.lines.iter_mut() {
                line.comment_opt = None;
            }
        }
        // Remove extraneous instructions
        let mut i = 0;
        while i < mips.lines.len() {
            let Line { stmt, comment_opt } = &mut mips.lines[i];
            match stmt {
                Stmt::Empty(..)
                    if (conf.remove_empty
                        && (conf.remove_empty_comments || comment_opt.is_none())) =>
                {
                    mips.remove_line(i);
                }
                Stmt::Define([Arg::String(key), ..], ..) if conf.remove_defines => {
                    mips.present_aliases.remove(key);
                    mips.remove_line(i);
                }
                Stmt::Alias([Arg::String(key), ..], ..) if conf.remove_aliases => {
                    mips.present_aliases.remove(key);
                    mips.remove_line(i);
                }
                Stmt::Tag([Arg::String(key)], ..) if conf.remove_tags => {
                    let alias = Alias::Num(i as f64);
                    mips.aliases.insert(key.clone(), alias);
                    mips.present_aliases.remove(key);
                    mips.remove_line(i);
                }
                _ => i += 1,
            }
        }

        // Re-lex
        mips.lex_lines()?;

        // Reduce instructions
        let mut lines = mips.lines.clone();
        for line in lines.iter_mut() {
            line.stmt.reduce_args(&mips).ok();
        }
        mips.lines = lines;

        if conf.optimize_registers {
            let lifetimes = self.analyze_lifetimes();
            let n = lifetimes.len();
            let node_iter = lifetimes
                .iter()
                .map(|Lifetime { index, .. }| *index)
                .collect::<BTreeSet<usize>>()
                .into_iter()
                .map(|i| (i, i));
            let edge_iter = (0..n)
                .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
                .filter_map(|(i, j)| {
                    let Lifetime {
                        index: i,
                        s: i_s,
                        e: i_e,
                    } = lifetimes[i];
                    let Lifetime {
                        index: j,
                        s: j_s,
                        e: j_e,
                    } = lifetimes[j];
                    ((i_s < j_e) && (j_s < i_e)).then_some((i, j))
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
                    if let Some(RegBase::Lit(reg_lit)) = arg.as_reg_base_mut() {
                        if let Some(new_index) = colors.get(&reg_lit.index) {
                            reg_lit.index = *new_index;
                        }
                    }
                }
            }
            //             // for reg in self.registers.iter_mut() {
            //             //     let mut reg_raw = reg.0.borrow_mut();
            //             //     let index = colors[&reg_raw.index];
            //             //     reg_raw.index = index;
            //             // }

            //             // let n = self.registers.len();
            //             // let node_iter = (0..n).map(|i| {
            //             //     let i = self.registers[i].index();
            //             //     (i, i)
            //             // });
            //             // let edge_iter = (0..n)
            //             //     .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
            //             //     .filter_map(|(i, j)| {
            //             //         let (i_s, i_e) = self.registers[i].lifetime();
            //             //         let (j_s, j_e) = self.registers[j].lifetime();

            //             //         ((i_s != i_e) && (j_s != j_e) && (i_s < j_e) && (j_s < i_e)).then_some({
            //             //             let i = self.registers[i].index();
            //             //             let j = self.registers[j].index();
            //             //             (i, j)
            //             //         })
            //             //     });

            //             // let graph = Graph::from_edges(node_iter.chain(edge_iter)).color();

            //             // TODO: simplify the graph before coloring

            //             // let colors = graph
            //             //     .into_nodes()
            //             //     .map(|node| {
            //             //         let index = node.index();
            //             //         let color = node.color().unwrap();
            //             //         (index, color)
            //             //     })
            //             //     .collect::<BTreeMap<_, _>>();

            //             // for reg in self.registers.iter_mut() {
            //             //     let mut reg_raw = reg.0.borrow_mut();
            //             //     let index = colors[&reg_raw.index];
            //             //     reg_raw.index = index;
            //             // }
        }
        Ok(mips)
    }

    /// Remove a line safely
    ///
    /// Removes a line safely by keeping in mind edge cases:
    /// * adjusting relative branches that encompass the line to remove
    pub fn remove_line(&mut self, i: usize) {
        // Adjust relative branches encompassing the line to remove
        for (j, line) in self.lines.iter_mut().enumerate().filter(|&(j, _)| i != j) {
            for arg in line.stmt.iter_args_mut() {
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
        self.lines.remove(i);
    }

    // TODO: Add way to fix certain registers
    //  * automatically? see an indirect reg or dev -> fix the x of dr..rx or r..rx?
    //  * manually?
    pub fn analyze_lifetimes(&self) -> Vec<Lifetime> {
        let mut res = Vec::<Lifetime>::new();
        let mut lifetimes = BTreeMap::<usize, Lifetime>::new();
        let mut res_lookup = BTreeMap::<usize, Vec<usize>>::new();

        for (i, line) in self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| !matches!(line.stmt, Stmt::Alias(..)))
        {
            for (arg, reg_lit) in line.stmt.args().iter().filter_map(|arg| {
                arg.get_reg_lit(self)
                    .expect(&format!("Fatal error analyzing lifetimes\nARG: {:?}", arg))
                    .map(|reg_lit| (arg, reg_lit))
            }) {
                let index = reg_lit.index;
                if matches!(arg, Arg::Reg(..)) {
                    // As l-value
                    let (s, e) = if reg_lit.fixed {
                        // (0, self.units.len())
                        (i, i) // TODO: do something smarter
                    } else {
                        (i, i)
                    };
                    let lifetime = Lifetime { index, s, e };
                    if let Some(old_lifetime) = lifetimes.insert(index, lifetime) {
                        // If a lifetime for this register index is being replaced...

                        // Push the old lifetime to the results
                        let id = res.len();
                        res.push(old_lifetime);

                        // And add a lookup for the index to the position in results
                        res_lookup.entry(index).or_insert(Vec::new()).push(id);
                    }
                } else {
                    // As r-value
                    let Lifetime { e, .. } =
                        lifetimes
                            .entry(index)
                            .or_insert(Lifetime { index, s: i, e: i });
                    if *e < i {
                        *e = i;
                    }
                }
            }
        }
        res.extend(lifetimes.into_values());
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
        for (i, Lifetime { index, s, e }) in lifetimes.into_iter().enumerate() {
            let index_s = format!("r{}", index);
            output.push_str(&format!("{:>3} {:>3} :", i, index_s));
            //         let (s, e) = reg.lifetime();
            for i in 0..n {
                #[rustfmt::skip]
                match (s == i, s <= i && i <= e, i == e) {
                    ( true,  true,  true) => output.push_str(&format!("{:>2}", i % 10)),
                    ( true,  true, false) => output.push_str(&format!("{:>2}", i % 10)),
                    (false,  true, false) => output.push_str("--"),
                    (false,  true,  true) => output.push_str(&format!("{:->2}", i % 10)),
                    _                     => output.push_str(" |"),
                };
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
    pub remove_aliases: bool,
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
            remove_aliases: true,
            remove_defines: true,
            remove_tags: true,
            optimize_registers: true,
        }
    }
}
