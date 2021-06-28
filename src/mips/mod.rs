use std::collections::{BTreeMap, BTreeSet};

use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};

use crate::ast_traits::{AstNode, AstPair, AstPairs, IntoAst};
use crate::graph::Graph;
// use crate::mips::ast::IntoMipsNode;

#[derive(Parser)]
#[grammar = "mips/grammar.pest"]
pub struct MipsParser;

pub type Pair<'i> = pest::iterators::Pair<'i, Rule>;
pub type Pairs<'i> = pest::iterators::Pairs<'i, Rule>;

impl<'i> AstNode<'i, Rule, MipsParser, MipsError> for String {
    type Output = String;

    const RULE: Rule = Rule::alias;

    fn try_from_pair(pair: Pair) -> MipsResult<String> {
        Ok(pair.as_str().to_owned())
    }
}

pub mod ast;
use ast::{Arg, Dev, DevBase, DevLit, LineAbs, LineRel, MipsNode, Num, Reg, RegBase, Unit};

mod error;
pub use error::MipsError;

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
    pub units: Vec<Unit>,
    pub next_unique_id: usize,
}

impl Default for Mips {
    fn default() -> Self {
        // let registers = Vec::new();
        // let labels = BTreeMap::new();
        let aliases = BTreeMap::new();
        let present_aliases = BTreeSet::new();
        let units = Vec::new();
        Self {
            // registers,
            // labels,
            aliases,
            present_aliases,
            units,
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
    fn get_alias(&self, key: &str) -> MipsResult<&Alias> {
        self.aliases.get(key).ok_or(MipsError::alias_undefined(key))
    }

    fn has_alias(&self, key: &str) -> MipsResult<()> {
        self.get_alias(key).and(Ok(()))
    }

    fn get_dev_base(&self, key: &str) -> MipsResult<Option<DevBase>> {
        match self.get_alias(key)? {
            Alias::Dev(dev_lit) => Ok(Some(dev_lit.clone())),
            _ => Ok(None),
        }
    }

    fn get_only_dev_base(&self, key: &str) -> MipsResult<DevBase> {
        let alias = self.get_alias(key)?;
        match alias {
            Alias::Dev(dev_lit) => Ok(dev_lit.clone()),
            _ => Err(MipsError::alias_wrong_kind("a device", alias)),
        }
    }

    fn get_reg_base(&self, key: &str) -> MipsResult<Option<RegBase>> {
        match self.get_alias(key)? {
            Alias::Reg(reg_lit) => Ok(Some(reg_lit.clone())),
            _ => Ok(None),
        }
    }

    fn get_only_reg_base(&self, key: &str) -> MipsResult<RegBase> {
        let alias = self.get_alias(key)?;
        match alias {
            Alias::Reg(reg) => Ok(reg.clone()),
            _ => Err(MipsError::alias_wrong_kind("a register", alias)),
        }
    }

    fn has_reg_base(&self, key: &str) -> MipsResult<()> {
        self.get_reg_base(key).and(Ok(()))
    }

    fn get_num_lit(&self, key: &str) -> MipsResult<f64> {
        let alias = self.get_alias(key)?;
        match alias {
            Alias::Num(n) => Ok(*n),
            _ => Err(MipsError::alias_wrong_kind("a number", alias)),
        }
    }

    fn has_num_lit(&self, key: &str) -> MipsResult<()> {
        self.get_num_lit(key).and(Ok(()))
    }

    pub fn new(source: &str) -> Result<Self, String> {
        let mut mips = Mips::default();
        mips.parse_source(source)?;
        mips.lex_units()?;
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

            let unit = line_pair.try_into_ast::<Unit>().map_err(|err| {
                format!(
                    "Instruction parsing error\nLine: \"{}\"\nError: {}",
                    line, err
                )
            })?;

            self.units.push(unit);
        }
        Ok(())
    }

    fn lex_units(&mut self) -> Result<(), String> {
        // Line tag pass
        for (i, unit) in self.units.iter().enumerate() {
            if let Unit::Tag([Arg::String(tag)], _) = unit {
                self.aliases.insert(tag.clone(), Alias::Num(i as f64));
                self.present_aliases.insert(tag.clone());
            }
        }
        // Variable pass
        for i in 0..self.units.len() {
            self.lex_unit(i).map_err(|err| {
                let unit = &self.units[i];
                format!("Lexer error on line {}\nLine: {}\nError: {}", i, unit, err)
            })?;
        }
        Ok(())
    }

    fn lex_unit(&mut self, i: usize) -> MipsResult<()> {
        // Check that aliases are present
        for arg in self.units[i].iter_args() {
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
        match &self.units[i] {
            Unit::Alias([Arg::String(key), Arg::Reg(reg)], _) => {
                let reg_base = match reg {
                    Reg::Base(reg_base) => reg_base.clone(),
                    Reg::Alias(key) => self.get_only_reg_base(key)?,
                };
                let alias = Alias::Reg(reg_base);
                self.aliases.insert(key.clone(), alias);
                self.present_aliases.insert(key.clone());
            }
            Unit::Alias([Arg::String(key), Arg::Dev(dev)], _) => {
                let dev_base = match dev {
                    Dev::Base(dev_base) => dev_base.clone(),
                    Dev::Alias(key) => self.get_only_dev_base(key)?,
                };
                let alias = Alias::Dev(dev_base);
                self.aliases.insert(key.clone(), alias);
                self.present_aliases.insert(key.clone());
            }
            Unit::Define([Arg::String(key), Arg::Num(Num::Lit(n))], _) => {
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
            for unit in mips.units.iter_mut() {
                *unit.comment_mut() = None;
            }
        }
        // Remove extraneous instructions
        let mut i = 0;
        while i < mips.units.len() {
            let unit = &mut mips.units[i];
            match unit {
                Unit::Empty(..)
                    if (conf.remove_empty && (conf.remove_empty_comments || !unit.has_comment())) =>
                {
                    mips.remove_unit(i);
                }
                Unit::Define([Arg::String(key), ..], ..) if conf.remove_defines => {
                    mips.present_aliases.remove(key);
                    mips.remove_unit(i);
                }
                Unit::Alias([Arg::String(key), ..], ..) if conf.remove_aliases => {
                    mips.present_aliases.remove(key);
                    mips.remove_unit(i);
                }
                Unit::Tag([Arg::String(key)], ..) if conf.remove_tags => {
                    let alias = Alias::Num(i as f64);
                    mips.aliases.insert(key.clone(), alias);
                    mips.present_aliases.remove(key);
                    mips.remove_unit(i);
                }
                // Unit::Define([Arg::String(_), ..], _) if conf.remove_defines => {
                //     // let alias = mips.aliases[a].clone();
                //     // mips.aliases.insert(a.clone(), alias);
                //     mips.units.remove(i);
                // }
                _ => i += 1,
            }
        }

        //             // Rebuild aliases
        //             self.aliases.clear();
        //             for (key, alias) in self.units.iter().filter_map(Unit::alias_from) {
        //                 self.aliases.insert(key, alias);
        //             }

        //             // Re-analyze lifetimes
        //             // for (i, unit) in self.units.iter_mut().enumerate() {
        //             //     let mut iter = unit.iter_args_mut();
        //             //     if let Some(arg) = iter.next() {
        //             //         arg.update_lifetime(Some(i), Some(i));
        //             //     }
        //             //     while let Some(arg) = iter.next() {
        //             //         arg.update_lifetime(None, Some(i));
        //             //     }
        //             // }

        mips.lex_units()?;

        // Reduce instructions
        let mut units = mips.units.clone();
        for unit in units.iter_mut() {
            unit.reduce_args(&mips).ok();
        }
        mips.units = units;

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
                    // * a lifetime *cannot* conflict if it exists for only one unit (s == e)
                    // * lifetimes conflict if they overlap beyond start and ends
                    // ((i_s != i_e) && (j_s != j_e) && (i_s < j_e) && (j_s < i_e)).then_some((i, j))
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
            for unit in mips.units.iter_mut() {
                for arg in unit.iter_args_mut() {
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

    /// Remove a unit safely
    ///
    /// Removes a unit safely by keeping in mind edge cases:
    /// * adjusting relative branches that encompass the unit to remove
    pub fn remove_unit(&mut self, i: usize) {
        // Adjust relative branches encompassing the line to remove
        for (j, unit) in self.units.iter_mut().enumerate().filter(|&(j, _)| i != j) {
            for arg in unit.iter_args_mut() {
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
        self.units.remove(i);
    }

    // TODO: Add way to fix certain registers
    //  * automatically? see an indirect reg or dev -> fix the x of dr..rx or r..rx?
    //  * manually?
    pub fn analyze_lifetimes(&self) -> Vec<Lifetime> {
        let mut res = Vec::<Lifetime>::new();
        let mut lifetimes = BTreeMap::<usize, Lifetime>::new();
        for (i, unit) in self
            .units
            .iter()
            .enumerate()
            .filter(|(_, unit)| !matches!(unit, Unit::Alias(..)))
        {
            for (arg, reg_lit) in unit.args()
                .iter()
                .filter_map(|arg| {
                    arg.get_reg_lit(self)
                        .expect(&format!("Fatal error analyzing lifetimes\nARG: {:?}", arg))
                        .map(|reg_lit| (arg, reg_lit))
                })
            {
                let index = reg_lit.index;
                if matches!(arg, Arg::Reg(..)) {
                    // As l-value
                    let (s, e) = if reg_lit.fixed {
                        (0, self.units.len())
                    } else {
                        (i, i)
                    };
                    let lifetime = Lifetime { index, s, e };
                    if let Some(mut old_lifetime) = lifetimes.insert(index, lifetime) {
                        res.push(old_lifetime);
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
            // for reg_lit in args
            //     .iter()
            //     .filter(|arg| !matches!(arg, Arg::Reg(..)))
            //     .filter_map(|arg| {
            //         arg.get_reg_lit(self)
            //             .expect(&format!("Fatal error analyzing lifetimes\nARG: {:?}", arg))
            //     })
            // {
            //     let index = reg_lit.index;
            //     if !reg_lit.fixed {
            //         let Lifetime { e, .. } =
            //             lifetimes
            //                 .entry(index)
            //                 .or_insert(Lifetime { index, s: i, e: i });
            //         // .expect(&format!("Fatal error analyzing lifetimes\nMissing lifetime for index {}", index));
            //         *e = i;
            //     }
            // }
            // if let Some(Arg::Reg(reg)) = args.first() {
            //     if let Some(RegBase::Lit(reg_lit)) = reg
            //         .get_reg_base(self)
            //         .expect(&format!("Fatal error analyzing lifetimes\nREG: {:?}", reg))
            //     {
            //         let index = reg_lit.index;
            //         let (s, e) = if reg_lit.fixed {
            //             (0, self.units.len())
            //         } else {
            //             (i, i)
            //         };
            //         let lifetime = Lifetime { index, s, e };
            //         if let Some(mut old_lifetime) = lifetimes.insert(index, lifetime) {
            //             // old_lifetime.e = i;
            //             res.push(old_lifetime);
            //         }
            //     }
            // }
        }
        res.extend(lifetimes.into_values());
        res
    }

    pub fn interference_graph(&self) -> String {
        let lifetimes = self.analyze_lifetimes();

        let mut output = "LIFETIMES:\n         ".to_owned();
        let n = self.units.len();
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
