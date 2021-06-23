use std::collections::BTreeMap;

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::ast_traits::OnlyInner;
use crate::graph::Graph;

#[derive(Parser)]
#[grammar = "mips/grammar.pest"]
pub struct MipsParser;

pub mod ast;
use ast::{Arg, MipsNode, Num, Reg, Unit, Var};

mod error;
pub use error::MipsError;

pub type MipsResult<T> = Result<T, MipsError>;
#[derive(Clone, Debug)]
pub enum Alias {
    Var(Var),
    Num(Num),
}

impl Alias {
    pub fn reduce(self) -> Self {
        match self {
            Self::Var(var) => Self::Var(var.reduce()),
            Self::Num(num) => Self::Num(num.reduce()),
        }
    }

    pub fn reg(&self) -> Option<&Reg> {
        match self {
            Self::Var(var) => Some(var.reg()),
            Self::Num(num) => num.reg(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mips {
    pub registers: Vec<Reg>,
    pub aliases: BTreeMap<String, Alias>,
    pub units: Vec<Unit>,
    pub next_unique_id: usize,
}

impl Default for Mips {
    fn default() -> Self {
        let registers = Vec::new();
        let units = Vec::new();
        let aliases = BTreeMap::new();
        Self {
            registers,
            aliases,
            units,
            next_unique_id: 0,
        }
    }
}

impl Mips {
    pub(crate) fn new_reg(&mut self, index: usize, fixed: bool) -> Reg {
        let now = self.units.len();
        let reg = Reg::new(self.next_unique_id, index, fixed, now);
        self.next_unique_id += 1;
        self.registers.push(reg.clone());
        reg
    }

    // pub(crate) fn next_reg(&mut self, fixed: bool) -> Reg {
    //     self.new_reg(self.registers.len(), fixed)
    // }

    // pub fn next_reg(&mut self) -> Reg {
    //     let id = self.registers.last().map_or(0, |reg| reg.index() + 1);
    //     let reg = Reg::new(id, self.units.len());
    //     self.registers.push(reg.clone());
    //     reg
    // }

    pub fn get_alias(&self, k: &str) -> MipsResult<&Alias> {
        let opt = self.aliases.get(k);
        opt.ok_or(MipsError::alias_undefined(k))
    }

    pub fn get_reg(&self, k: &str) -> MipsResult<Reg> {
        if let Alias::Var(var) = self.get_alias(k)? {
            Ok(var.reg().clone())
        } else {
            Err(MipsError::AliasWrongKind)
        }
    }

    // pub fn get_reg_or_new(&mut self, k: &str) -> Reg {
    //     self.get_reg(k).unwrap_or_else(|_| self.next_reg(false))
    // }

    pub fn get_num(&self, k: &str) -> MipsResult<Num> {
        let alias = self.get_alias(k)?;
        let num = match alias {
            Alias::Num(n) => n.clone(),
            Alias::Var(v) => Num::Var(v.clone()),
        };
        // println!("found {:?}", num);
        // let now = self.units.len();
        // num.update_lifetime(None, Some(now));
        Ok(num)
    }

    pub fn try_ast_from_pair<'i, A>(&mut self, pair: Pair<Rule>) -> MipsResult<A>
    where
        A: MipsNode<'i, Rule, MipsParser, Output = A>,
    {
        A::try_from_pair(self, pair)
    }

    pub fn intepret_line(&mut self, line_str: &str) -> MipsResult<()> {
        // println!("line \"{}\"", line_str);
        let line_pairs = MipsParser::parse(Rule::line, line_str).map_err(|err| {
            MipsError::LineError(format!(
                "Failed to parse pair as line
Line: \"{}\"
Error: {:?}",
                line_str, err
            ))
        })?;

        let line_pair = line_pairs.only_inner().map_err(|err| {
            MipsError::LineError(format!(
                "Too many inner pairs for line pair
Line: \"{}\"
Error: {:?}",
                line_str, err
            ))
        })?;

        let unit: Unit = self.try_ast_from_pair(line_pair).map_err(|err| {
            MipsError::LineError(format!(
                "Error in line
Line: \"{}\"
Error: {}",
                line_str, err
            ))
        })?;

        self.push_unit(unit);

        Ok(())
    }

    pub fn push_unit(&mut self, mut unit: Unit) {
        let alias_from = unit.alias_from();
        println!("{} {:?}", unit, alias_from);
        if let Some((name, alias)) = alias_from {
            // match &alias
        //     let reg_name = alias.reg().map(Reg::to_string);
        //     // println!("ALIAS {:?}", alias);
            // self.aliases.insert(name.clone(), alias);
            self.aliases.insert(name, alias);
        //     if let Some(reg_name) = reg_name {
        //         let alias = self.aliases.get(&name).unwrap().clone();
        //         self.aliases.insert(reg_name, alias);
        //     }
        }
        // self.add_aliases_from_unit(&unit);

        let now = self.units.len();
        match unit.args_mut() {
            [Arg::Var(var), ..] => {
                var.update_lifetime(Some(now), None);
            }
            _ => {}
        }

        for arg in unit.iter_args_mut() {
            arg.update_lifetime(None, Some(now));
        }
        self.units.push(unit);
    }

    pub fn deep_clone(&self) -> Self {
        let mut mips = Mips::default();
        for reg in self.registers.iter() {
            mips.new_reg(reg.index(), reg.fixed());
        }
        for reg in mips.registers.iter() {
            let name = reg.to_string();
            let indirections = 0;
            let reg = reg.clone();
            let alias = Alias::Var(Var::Reg { indirections, reg });
            mips.aliases.insert(name, alias);
        }
        for (name, alias) in self.aliases.iter() {
            let mut alias = alias.clone();
            match &mut alias {
                Alias::Num(Num::Var(var)) | Alias::Var(var) => {
                    match var {
                        Var::Reg { reg, .. } | Var::Alias { reg, .. } => {
                            let k = reg.to_string();
                            let alias = mips.aliases.get(&k).unwrap().clone();
                            let new_reg = alias.reg().unwrap().clone();
                            *reg = new_reg;
                        }
                    }
                },
                _ => {},
            }
            mips.aliases.insert(name.clone(), alias);
        }
        mips
    }

    fn rebuild_aliases(&mut self) {
        self.aliases.clear();
        // TODO: Use unsafe to have multiple mut borrows
        for (name, alias) in self.units.iter().filter_map(Unit::alias_from) {
            self.aliases.insert(name, alias);
        }
    }

    // fn rebuild_lifetimes(&mut self) {
    // }

    // TODO: Have optimize functions return a new object.
    // Need to implement deep copy on Mips
    pub fn optimize_instructions(&mut self) {
        // Remove extraneous instructions
        let mut i = 0;
        while i < self.units.len() {
            let unit = &mut self.units[i];
            // let s = format!("{:?}", unit);
            match unit {
                Unit::Alias(..) => {
                    // println!("removed {}", s);
                    self.units.remove(i);
                    self.registers.remove(i);
                }
                Unit::Define([Arg::String(a), ..], _) => {
                    // println!("removed {}", s);
                    let alias = self.aliases[a].clone();
                    self.aliases.insert(a.clone(), alias);
                    self.units.remove(i);
                }
                Unit::Empty(..) => {
                    // println!("removed {}", s);
                    self.units.remove(i);
                }
                _ => {
                    unit.reduce_args();
                    i += 1;
                }
            }
        }
        // Remove unused aliases
        // let unique_ids = self
        //     .registers
        //     .iter()
        //     .map(|reg| reg.unique_id())
        //     .collect::<Vec<_>>();
        // println!("unique ids {:?}", unique_ids);
        // self.aliases.retain(|_, alias| {
        //     println!("{:?}", alias);
        //     if let Alias::Var(Var::Reg { reg, .. }) = alias {
        //         let unique_id = reg.unique_id();
        //         // println!("{}", unique_id);
        //         // println!("{:?}", reg);
        //         // unique_ids.contains(&unique_id)
        //         true
        //     } else {
        //         false
        //     }
        // });
        // Rebuild aliases
        self.rebuild_aliases();
        // Re-analyze lifetimes
        for (i, unit) in self.units.iter_mut().enumerate() {
            let mut iter = unit.iter_args_mut();
            if let Some(arg) = iter.next() {
                arg.update_lifetime(Some(i), Some(i));
            }
            while let Some(arg) = iter.next() {
                arg.update_lifetime(None, Some(i));
            }
        }
    }

    pub fn optimize_registers(&mut self) {
        let n = self.registers.len();

        let node_iter = (0..n).map(|i| {
            let i = self.registers[i].index();
            (i, i)
        });
        let edge_iter = (0..n)
            .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
            .filter_map(|(i, j)| {
                let (i_s, i_e) = self.registers[i].lifetime();
                let (j_s, j_e) = self.registers[j].lifetime();

                ((i_s != i_e) && (j_s != j_e) && (i_s < j_e) && (j_s < i_e)).then_some({
                    let i = self.registers[i].index();
                    let j = self.registers[j].index();
                    (i, j)
                })
            });

        let graph = Graph::from_edges(node_iter.chain(edge_iter)).color();

        // TODO: simplify the graph before coloring

        let colors = graph
            .into_nodes()
            .map(|node| {
                let index = node.index();
                let color = node.color().unwrap();
                (index, color)
            })
            .collect::<BTreeMap<_, _>>();

        for reg in self.registers.iter_mut() {
            let mut reg_raw = reg.0.borrow_mut();
            let index = colors[&reg_raw.index];
            reg_raw.index = index;
        }

        self.rebuild_aliases();
    }

    pub fn interference_graph(&self) -> String {
        let n = self.units.len();
        let mut output = "LIFETIMES:\n         ".to_owned();
        for i in 0..n {
            if i % 10 == 0 {
                output.push_str(&format!("{:>2}", i));
            } else {
                output.push_str("  ");
            }
        }
        output.push_str("\n");
        for reg in self.registers.iter() {
            output.push_str(&format!("{:>3} {:>3} : ", reg.unique_id(), reg));
            let (s, e) = reg.lifetime();
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
