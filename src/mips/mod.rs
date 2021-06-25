use std::collections::BTreeMap;

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
use ast::{Arg, LineAbs, LineRel, Num, Reg, Unit};

mod error;
pub use error::MipsError;

pub type MipsResult<T> = Result<T, MipsError>;

#[derive(Clone, Debug)]
pub enum Alias {
    Num(f64),
    //     // Dev
    Reg(Reg),
}

// impl Alias {
//     // pub fn reduce(self) -> Self {
//     //     match self {
//     //         Self::Reg(reg) => Self::Reg(reg.reduce()),
//     //         Self::Num(num) => Self::Num(num.reduce()),
//     //     }
//     // }

//     pub fn reg_raw(&self) -> Option<&RegRaw> {
//         match self {
//             Self::Reg(reg) => Some(reg.reg_raw()),
//             _ => None,
//         }
//     }

//     pub fn reg_raw_mut(&mut self) -> Option<&mut RegRaw> {
//         match self {
//             Self::Reg(reg) => Some(reg.reg_raw_mut()),
//             _ => None,
//         }
//     }
// }

#[derive(Clone, Debug)]
pub struct Mips {
    //     // pub registers: Vec<RegShared>,
    pub labels: BTreeMap<String, usize>,
    pub aliases: BTreeMap<String, Alias>,
    pub units: Vec<Unit>,
    pub next_unique_id: usize,
}

impl Default for Mips {
    fn default() -> Self {
        // let registers = Vec::new();
        let units = Vec::new();
        let labels = BTreeMap::new();
        let aliases = BTreeMap::new();
        Self {
            // registers,
            labels,
            aliases,
            units,
            next_unique_id: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Lifetime {
    unique_id: usize,
    index: usize,
    s: usize,
    e: usize,
}

impl Mips {
    //     pub(crate) fn new_reg_raw(&mut self, index: usize, fixed: bool) -> RegRaw {
    //         let reg_raw = RegRaw::new(self.next_unique_id, index, fixed);
    //         self.next_unique_id += 1;
    //         reg_raw
    //     }

    //     pub fn get_alias(&self, k: &str) -> MipsResult<&Alias> {
    //         let opt = self.aliases.get(k);
    //         opt.ok_or(MipsError::alias_undefined(k))
    //     }

    //     pub fn get_reg(&self, k: &str) -> MipsResult<&Reg> {
    //         let alias = self.get_alias(k)?;
    //         match alias {
    //             Alias::Reg(reg) => Ok(reg),
    //             _ => Err(MipsError::alias_wrong_kind("a register", alias)),
    //         }
    //     }

    //     pub fn get_reg_raw(&self, k: &str) -> MipsResult<&RegRaw> {
    //         Ok(self.get_reg(k)?.reg_raw())
    //     }

    //     pub fn get_num(&self, k: &str) -> MipsResult<Num> {
    //         let alias = self.get_alias(k)?;
    //         match alias {
    //             Alias::NumLit(n) => Ok(Num::Lit(*n)),
    //             Alias::Reg(reg_shared) => Ok(Num::Reg(reg_shared.clone().into())),
    //             // Alias::LineNum(line_num) =>
    //             _ => Err(MipsError::alias_wrong_kind("a number", alias)),
    //         }
    //     }

    //     // pub fn get_line_num(&self, k: &str) -> MipsResult<LineNum> {
    //     //     let alias = self.get_alias(k)?;
    //     //     match alias {

    //     //     }
    //     // }

    pub fn new(source: &str) -> Result<Self, String> {
        let mut mips = Self::interpret_str(source)?;
        mips.update()?;
        Ok(mips)
    }

    fn interpret_str(source: &str) -> Result<Self, String> {
        let mut mips = Self::default();

        for line in source.trim_end().split("\n") {
            let pairs = MipsParser::parse(Rule::line, line).map_err(|err| {
                format!(
                    "Failed to parse pair as line\nLine: \"{}\"\nError: {:?}",
                    line, err
                )
            })?;

            let line_pair = pairs.only_pair().map_err(|err| {
                format!(
                    "Too many inner pairs for line pair\nLine: \"{}\"\nError: {:?}",
                    line, err
                )
            })?;

            let unit = line_pair
                .try_into_ast::<Unit>()
                .map_err(|err| format!("Error in line\nLine: \"{}\"\nError: {}", line, err))?;

            mips.units.push(unit);
        }

        Ok(mips)
    }

    fn update(&mut self) -> Result<(), String> {
        // Label pass
        for (i, unit) in self.units.iter().enumerate() {
            if let Unit::Label([Arg::String(label)], _) = unit {
                self.labels.insert(label.clone(), i);
            }
        }
        // Variable pass
        for (i, unit) in self.units.iter().enumerate() {}
        Ok(())
    }

    //     pub fn set_alias_from_unit(&mut self, unit: &Unit) {
    //         if let Some((key, alias)) = unit.alias_from() {
    //             if let Some(rs) = alias.reg_shared() {
    //                 let indirections = 0;
    //                 let reg_shared = rs.clone();
    //                 let reg = Reg::Lit {
    //                     indirections,
    //                     reg_shared,
    //                 };
    //                 let _alias = Alias::Reg(reg);
    //                 // self.aliases.insert(rs.to_string(), alias);
    //             }
    //             self.aliases.insert(key, alias);
    //         }
    //     }

    //     pub fn push_unit(&mut self, unit: Unit) {
    //         // Add alias if unit is an assignment
    //         self.set_alias_from_unit(&unit);
    //         self.units.push(unit);
    //     }

    //     pub fn deep_copy(&self) -> Self {
    //         let mut mips = Mips::default();
    //         // Add registers
    //         // for reg in self.registers.iter() {
    //         //     mips.registers.push(reg.deep_copy());
    //         // }
    //         // Add base aliases for registers
    //         // for reg_shared in mips.registers.iter() {
    //         //     let alias = Alias::Reg(reg_shared.clone().into());
    //         //     mips.aliases.insert(reg_shared.to_string(), alias);
    //         // }
    //         // Add units
    //         // for mut unit in self.units.iter().cloned() {
    //         //     unit.update_regs(&mips);
    //         //     mips.push_unit(unit);
    //         // }
    //         mips
    //     }

    //     pub fn analyze_lifetimes(&self) -> Vec<Lifetime> {
    //         let mut lifetimes = BTreeMap::new();
    //         for (i, unit) in self.units.iter().enumerate() {
    //             match unit {
    //                 Unit::Alias([_, Arg::Reg(_reg)], _) => {
    //                     // let unique_id = _reg.unique_id();
    //                     // let index = _reg.index();
    //                     // lifetimes.insert(unique_id, (index, i, i));
    //                 },
    //                 _ => {
    //                     let mut arg_iter = unit.iter_args();
    //                     if let Some(Arg::Reg(reg)) = arg_iter.next() {
    //                         let unique_id = reg.unique_id();
    //                         let index = reg.index();
    //                         lifetimes.insert(unique_id, (index, i, i));
    //                     }
    //                     while let Some(arg) = arg_iter.next() {
    //                         if let Some(reg) = arg.reg() {
    //                             let unique_id = reg.unique_id();
    //                             let (_, _, e) = lifetimes.get_mut(&unique_id).unwrap();
    //                             *e = i;
    //                         }
    //                     }
    //                 },
    //             }
    //         }
    //         lifetimes.into_iter()
    //             .map(|(unique_id, (index, s, e))| Lifetime { unique_id, index, s, e })
    //             .collect()
    //     }

    /// Remove a unit safely
    ///
    /// Removes a unit safely by keeping in mind edge cases:
    /// * adjusting relative branches that encompass the unit to remove
    pub fn remove_unit(&mut self, i: usize) {
        self.units.remove(i);
        // Adjust relative branching
        for (j, unit) in self.units.iter_mut().enumerate() {
            for arg in unit.iter_args_mut() {
                if let Arg::LineRel(LineRel::Lit(n)) = arg {
                    let i = i as isize;
                    let j = j as isize;
                    if (*n < 0) && (j + *n <= i) { *n += 1; }
                    // if j + *n < i { *n += 1; }
                    // if j - *n >= i { *n += 1; }
                }
            }
        }
    }

    pub fn optimize(&mut self, conf: OptimizationConfig) {
        if conf.remove_or_reduce() {
            if conf.remove_comments {
                for unit in self.units.iter_mut() {
                    *unit.comment_mut() = None;
                }
            }

            // Remove extraneous instructions
            let mut i = 0;
            while i < self.units.len() {
                let unit = &mut self.units[i];
                match unit {
                    Unit::Empty(..) if (conf.remove_empty && !unit.has_comment()) => {
                        self.remove_unit(i);
                        // self.units.remove(i);
                    }
                    //                     Unit::Alias(..) if conf.reduce_aliases => {
                    //                         self.units.remove(i);
                    //                     }
                    //                     Unit::Define([Arg::String(_), ..], _) if conf.reduce_defines => {
                    //                         // let alias = self.aliases[a].clone();
                    //                         // self.aliases.insert(a.clone(), alias);
                    //                         self.units.remove(i);
                    //                     }
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
            //         }

            //         if conf.optimize_registers {
            //             println!("OPTIMIZE REGISTERS");
            //             let lifetimes = self.analyze_lifetimes();
            //             let n = lifetimes.len();
            //             let node_iter = (0..n)
            //                 .map(|i| {
            //                     let index = lifetimes[i].index;
            //                     (index, index)
            //                 });
            //             let edge_iter = (0..n)
            //                 .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
            //                 .filter_map(|(i ,j)| {
            //                     let Lifetime { index: i, s: i_s, e: i_e, .. } = lifetimes[i];
            //                     let Lifetime { index: j, s: j_s, e: j_e, .. } = lifetimes[j];
            //                     ((i_s != i_e) && (j_s != j_e) && (i_s < j_e) && (j_s < i_e)).then_some((i, j))
            //                 });
            //             let graph = Graph::from_edges(node_iter.chain(edge_iter)).color();
            //             let colors = graph
            //                 .into_nodes()
            //                 .map(|node| {
            //                     let index = node.index();
            //                     let color = node.color().unwrap();
            //                     (index, color)
            //                 })
            //                 .collect::<BTreeMap<_, _>>();
            //             for unit in self.units.iter_mut() {
            //                 for arg in unit.iter_args_mut() {
            //                     if let Some(mut reg_raw) = arg.reg_shared_mut().map(|rc| rc.borrow_mut()) {
            //                         if let Some(new_index) = colors.get(&reg_raw.index) {
            //                             reg_raw.index = *new_index;
            //                         }
            //                     }
            //                 }
            //             }
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
    }

    //     pub fn interference_graph(&self) -> String {
    //         let lifetimes = self.analyze_lifetimes();

    //         let mut output = "LIFETIMES:\n         ".to_owned();
    //         let n = self.units.len();
    //         for i in 0..n {
    //             if i % 10 == 0 {
    //                 output.push_str(&format!("{:>2}", i));
    //             } else {
    //                 output.push_str("  ");
    //             }
    //         }
    //         output.push_str("\n");
    //         for Lifetime { unique_id, index, s, e } in lifetimes {
    //     //     for reg in self.registers.iter() {
    //             let index_s = format!("r{}", index);
    //             output.push_str(&format!("{:>3} {:>3} :", unique_id, index_s));
    //     //         let (s, e) = reg.lifetime();
    //             for i in 0..n {
    //     //             #[rustfmt::skip]
    //                 match (s == i, s <= i && i <= e, i == e) {
    //                     ( true,  true,  true) => output.push_str(&format!("{:>2}", i % 10)),
    //                     ( true,  true, false) => output.push_str(&format!("{:>2}", i % 10)),
    //                     (false,  true, false) => output.push_str("--"),
    //                     (false,  true,  true) => output.push_str(&format!("{:->2}", i % 10)),
    //                     _                     => output.push_str(" |"),
    //                 };
    //             }
    //             output.push_str("\n");
    //         }
    //         output
    //     }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub remove_comments: bool,
    pub remove_empty: bool,
    pub reduce_aliases: bool,
    pub reduce_defines: bool,
    pub reduce_labels: bool,
    pub optimize_registers: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            remove_comments: true,
            remove_empty: true,
            reduce_aliases: true,
            reduce_defines: true,
            reduce_labels: true,
            optimize_registers: true,
        }
    }
}

impl OptimizationConfig {
    pub fn remove_or_reduce(&self) -> bool {
        self.remove_comments
            || self.remove_empty
            || self.reduce_aliases
            || self.reduce_defines
            || self.reduce_labels
    }
}
