use std::collections::BTreeMap;

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::ast_traits::{
    // NextPair,
    OnlyInner,
    // FinalPair,
};
use crate::graph::Graph;

#[derive(Parser)]
#[grammar = "mips/grammar.pest"]
pub struct MipsParser;

pub mod ast;
use ast::{Arg, AstError, AstNode, AstResult, Reg, RegBase, Unit, Num};

#[derive(Clone, Debug)]
pub enum Alias {
    Reg(Reg),
    Num(Num),
}

#[derive(Clone, Debug)]
pub struct Mips {
    pub registers: Vec<RegBase>,
    pub units: Vec<Unit>,
    pub aliases: BTreeMap<String, Alias>,
}

impl Default for Mips {
    fn default() -> Self {
        let registers = Vec::new();
        let units = Vec::new();
        let aliases = BTreeMap::new();
        Self {
            registers,
            units,
            aliases,
        }
    }
}

impl Mips {
    pub(crate) fn indexed_reg(&mut self, index: usize) -> RegBase {
        // let key = AliasKey::String(format!("r{}", index));
        if let Some(reg) = self.registers.iter().find(|reg| reg.index() == index) {
            reg.clone()
        } else {
            let reg = RegBase::new(index, 0, self.units.len());
            self.registers.insert(index, reg.clone());
            reg
        }
    }

    // pub fn next_reg(&mut self) -> Reg {
    //     let id = self.registers.last().map_or(0, |reg| reg.index() + 1);
    //     let reg = Reg::new(id, self.units.len());
    //     self.registers.push(reg.clone());
    //     reg
    // }

    pub fn get_alias<K: std::borrow::Borrow<str>>(&self, k: K) -> Option<Alias> {
        self.aliases.get(k.borrow()).cloned()
    }

    pub fn get_reg<K: std::borrow::Borrow<str>>(&self, k: K) -> Option<Reg> {
        if let Some(Alias::Reg(reg)) = self.get_alias(k) {
            Some(reg)
        } else {
            None
        }
    }

    pub fn push_unit(&mut self, mut unit: Unit) -> Option<Reg> {
        fn update_lifetimes(mips: &mut Mips, unit: &mut Unit) {
            let mut iter = unit.iter_args_mut();

            let now = mips.units.len();

            // if let Some(arg) = iter.next() {
            // while let Some(arg) = iter.next() {
            //     arg.update_lifetime(Some(now), Some(now));
            // }
            while let Some(arg) = iter.next() {
                arg.update_lifetime(None, Some(now));
            }
        }

        match unit {
            Unit::Alias(..) => {},
            _ => update_lifetimes(self, &mut unit),
        }

        let reg_opt = match unit.iter_args().next() {
            Some(Arg::Reg(reg)) => Some(reg.clone()),
            _ => None,
        };
        self.units.push(unit);
        reg_opt
    }

    pub fn try_ast_from_pair<'i, A>(&mut self, pair: Pair<Rule>) -> AstResult<A>
    where
        A: AstNode<'i, Rule, MipsParser, AstError, Output = A>,
    {
        A::try_from_pair(self, pair)
    }

    pub fn intepret_line(&mut self, line_str: &str) {
        let line_pair = MipsParser::parse(Rule::line, line_str)
            .unwrap()
            .only_inner()
            .unwrap();

        let unit: Unit = self.try_ast_from_pair(line_pair).unwrap();

        self.push_unit(unit);
    }

    pub fn optimize_registers(&mut self) -> Graph {
        let n = self.registers.len();

        let node_iter = (0..n).map(|i| (i, i));
        let edge_iter = (0..n)
            .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
            .filter_map(|(i, j)| {
                let (i_s, i_e) = self.registers[i].lifetime();
                let (j_s, j_e) = self.registers[j].lifetime();

                (i_s < j_e && j_s < i_e).then_some((i, j))
            });

        let graph = Graph::from_edges(node_iter.chain(edge_iter));

        // TODO: simplify the graph before coloring

        let colors = graph
            .color()
            .into_nodes()
            .map(|node| node.color().unwrap())
            .collect::<Vec<usize>>();

        for (reg, i) in self.registers.iter_mut().zip(colors.into_iter()) {
            reg.raw.borrow_mut().index = i;
        }

        graph
    }

    pub fn interference_graph(&self) -> String {
        let n = self.units.len();
        let mut output = "LIFETIMES:\n     ".to_owned();
        for i in 0..n {
            if i % 10 == 0 {
                output.push_str(&format!("{:>2}", i));
            } else {
                output.push_str("  ");
            }
        }
        output.push_str("\n     ");
        for i in 0..n {
            output.push_str(&format!("{:>2}", i));
        }
        output.push_str("\n");
        for reg in self.registers.iter() {
            output.push_str(&format!("{:<3} : ", reg));
            let (s, e) = reg.lifetime();
            for i in 0..n {
                #[rustfmt::skip]
                match (s == i, s <= i && i <= e, i == e) {
                    ( true,  true,  true) => output.push_str(&format!("{:>2}", i)),
                    ( true,  true, false) => output.push_str(&format!("{:>2}", i)),
                    (false,  true, false) => output.push_str("--"),
                    (false,  true,  true) => output.push_str(&format!("{:->2}", i)),
                    _                     => output.push_str(" |"),
                };
            }
            output.push_str("\n");
        }
        output
    }
}
