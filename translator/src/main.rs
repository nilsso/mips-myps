#![feature(box_patterns)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
#![allow(unused_assignments)]
#![allow(dead_code)]
use std::collections::BTreeMap;

use maplit::btreemap;

use mips::Mips;

#[derive(Clone, Debug)]
pub enum Item {
    Block(Vec<Item>),
    Unit(mips::ast::Unit),
}

// TODO: Flatten Items into Units
// impl Iterator for Item

// impl Item {
// }

#[derive(Clone, Debug)]
pub enum Alias {
    Reg(mips::ast::RegBase),
}

#[derive(Clone, Debug)]
pub struct Translator {
    units: Vec<mips::ast::Unit>,
    aliases: BTreeMap<String, Alias>,
    next_index: usize,
}

fn flatten_item(item: Item) -> Box<dyn Iterator<Item = mips::ast::Unit>> {
    match item {
        Item::Block(items) => Box::new(items.into_iter().flat_map(flatten_item)),
        Item::Unit(unit) => Box::new(std::iter::once(unit)),
    }
}

impl Translator {
    fn next_reg(&mut self, fixed: bool) -> mips::ast::RegBase {
        let reg_base = mips::ast::RegBase::new_lit(self.next_index, 0, fixed);
        self.next_index += 1;
        reg_base
    }

    fn get_alias(&self, key: &String) -> Option<&Alias> {
        self.aliases.get(key)
    }

    fn get_reg(&self, key: &String) -> mips::ast::RegBase {
        let alias = self.get_alias(key).unwrap();
        match alias {
            Alias::Reg(reg) => *reg,
        }
    }

    fn get_reg_or_new(&mut self, key: &String, fixed: bool) -> mips::ast::RegBase {
        if let Some(alias) = self.get_alias(key) {
            match alias {
                Alias::Reg(reg_base) => *reg_base,
            }
        } else {
            let reg_base = self.next_reg(fixed);
            self.aliases.insert(key.clone(), Alias::Reg(reg_base));
            reg_base
        }
    }

    fn unwrap_reg(&mut self, reg_opt: Option<mips::ast::RegBase>) -> mips::ast::RegBase {
        if let Some(reg_base) = reg_opt {
            reg_base
        } else {
            self.next_reg(false)
        }
    }

    pub fn translate_myps(program_item: myps::ast::Item) -> Self {
        let mut translator = Self {
            units: Vec::new(),
            aliases: btreemap! {
                "sp".into() => Alias::Reg(mips::ast::RegBase::SP),
                "ra".into() => Alias::Reg(mips::ast::RegBase::RA),
            },
            next_index: 0,
        };

        let program_item = translator.translate_item(program_item);

        let units = program_item
            .into_iter()
            .flat_map(flatten_item)
            .collect::<Vec<_>>();

        translator.units = units;
        translator
    }

    fn translate_item(&mut self, item: myps::ast::Item) -> Vec<Item> {
        // TODO: Need intermediate structure for Items
        use myps::ast::Item;

        match item {
            Item::Block(block, comment_opt) => {
                let item = self.translate_block(block, comment_opt);
                vec![item]
            }
            Item::Stmt(stmt, comment_opt) => self.translate_stmt(stmt, comment_opt),
        }
    }

    fn translate_block(&mut self, block: myps::ast::Block, comment_opt: Option<String>) -> Item {
        use mips::ast::{Arg, Num, Unit};
        use myps::ast::Branch;
        use std::iter::once;

        let myps::ast::Block { branch, items } = block;
        let items = items
            .into_iter()
            .flat_map(|item| self.translate_item(item))
            .collect::<Vec<_>>();
        let (head, tail) = self.translate_branch(branch, items.len(), comment_opt);
        let items = head.chain(items.into_iter()).chain(tail).collect();
        Item::Block(items)
    }

    fn translate_branch(
        &mut self,
        branch: myps::ast::Branch,
        num_items: usize,
        comment_opt: Option<String>,
    ) -> (
        Box<dyn Iterator<Item = Item>>,
        Box<dyn Iterator<Item = Item>>,
    ) {
        use mips::ast::{Arg, Num, Unit};
        use myps::ast::Branch;
        use std::iter;

        match branch {
            Branch::Program => (Box::new(iter::empty()), Box::new(iter::empty())),
            Branch::Loop => {
                let unit = Unit::Jr([Arg::Num((-(num_items as f64)).into())], comment_opt);
                let item = Item::Unit(unit);

                (Box::new(iter::empty()), Box::new(iter::once(item)))
            }
            Branch::Tag(tag) => {
                let unit = Unit::Tag([Arg::String(tag)], comment_opt);
                let item = Item::Unit(unit);

                (Box::new(iter::once(item)), Box::new(iter::empty()))
            }
            _ => unreachable!("{:?}", branch),
        }
    }

    fn translate_stmt(
        &mut self,
        stmt: myps::ast::Stmt,
        mut comment_opt: Option<String>,
    ) -> Vec<Item> {
        use myps::ast::Stmt;

        match stmt {
            Stmt::LvRvAsn(lvs, exprs) => {
                let mut units = Vec::new();
                for (lv, rv) in lvs.into_iter().zip(exprs) {
                    self.translate_assignment(&mut units, lv, rv, &mut comment_opt);
                }
                units.into_iter().map(|unit| Item::Unit(unit)).collect()
            }
            Stmt::SelfAsn { op, lhs, rhs } => {
                unreachable!();
            },
            Stmt::Empty => {
                let unit = mips::ast::Unit::Empty([], comment_opt);
                vec![Item::Unit(unit)]
            }
            Stmt::Mips(..) => {
                unreachable!();
            }
            // Unit::Func(..) => {
            //     self.translate(
            // }
        }
    }

    fn translate_assignment(
        &mut self,
        units: &mut Vec<mips::ast::Unit>,
        lv: myps::ast::Lv,
        rv: myps::ast::Rv,
        comment_opt: &mut Option<String>,
    ) {
        use mips::ast::{Arg, Reg};
        use myps::ast::{Lv, Var};

        match lv {
            Lv::Var(Var { key, fixed }) => {
                let reg = self.get_reg_or_new(&key, fixed);

                let mut comment_opt = comment_opt.take();
                if fixed {
                    comment_opt = comment_opt
                        .take()
                        .and_then(|mut comment| {
                            comment.push_str(" [FIX]");
                            Some(comment)
                        })
                        .or(Some("# [FIX]".to_owned()));
                }

                unreachable!();
                // let (num_d, num) = self.translate_expr(units, Some(reg), expr, &mut comment_opt);
                // if num_d == 0 {
                //     let r = Arg::Reg(Reg::Base(reg));
                //     let a = Arg::Num(num);
                //     let unit = mips::ast::Unit::Move([r, a], comment_opt);
                //     units.push(unit);
                // }
            }
            Lv::DevParam { dev, param } => {
                unreachable!();
            },
            Lv::NetParam { hash, param } => {
                unreachable!();
            }
        }
    }

    fn translate_expr(
        &mut self,
        units: &mut Vec<mips::ast::Unit>,
        reg: Option<mips::ast::RegBase>,
        expr: myps::ast::Expr,
        comment_opt: &mut Option<String>,
    ) -> (usize, mips::ast::Num) {
        use mips::ast::{Arg, Num, Reg, Unit};
        use myps::ast::Expr;

        match expr {
            Expr::Num(num) => {
                unreachable!();
                // self.translate_rv(units, rv)
            },
            Expr::Unary { op, rhs } => {
                unreachable!();
                // use myps::ast::UnaryOp;

                // let (b_d, b_num) = self.translate_expr(units, None, rhs, &mut None);

                // let reg = self.unwrap_reg(reg);
                // let r = Arg::Reg(Reg::Base(reg));
                // let b = Arg::Num(b_num);

                // let comment_opt = comment_opt.take();
                // #[rustfmt::skip]
                // let unit = match op {
                //     UnaryOp::Inv => {
                //         let a = Arg::Num(Num::Lit(0_f64));
                //         Unit::Sub([r, a, b], comment_opt)
                //     }
                //     UnaryOp::Not => {
                //         let a = b.clone();
                //         Unit::Nor([r, a, b], comment_opt)
                //     },
                // };
                // units.push(unit);

                // (b_d + 1, Num::Reg(reg))
            }
            Expr::Binary { op, lhs, rhs } => {
                use myps::ast::BinaryOp;

                unreachable!();

                // let (a_d, a_num) = self.translate_expr(units, None, lhs, &mut None);
                // let (b_d, b_num) = self.translate_expr(units, None, rhs, &mut None);

                // let reg = self.unwrap_reg(reg);
                // let r = Arg::Reg(Reg::Base(reg));
                // let a = Arg::Num(a_num);
                // let b = Arg::Num(b_num);
                // let comment_opt = comment_opt.take();
                // #[rustfmt::skip]
                // let unit = match op {
                //     // Numerical
                //     BinaryOp::Add => Unit::Add([r, a, b], comment_opt),
                //     BinaryOp::Sub => Unit::Sub([r, a, b], comment_opt),
                //     BinaryOp::Mul => Unit::Mul([r, a, b], comment_opt),
                //     BinaryOp::Div => Unit::Div([r, a, b], comment_opt),
                //     BinaryOp::Rem => Unit::Mod([r, a, b], comment_opt),
                //     // Logical
                //     BinaryOp::And => Unit::And([r, a, b], comment_opt),
                //     BinaryOp::Or  => Unit::Or ([r, a, b], comment_opt),
                //     BinaryOp::Xor => Unit::Xor([r, a, b], comment_opt),
                //     // Relational
                //     BinaryOp::EQ  => Unit::Seq([r, a, b], comment_opt),
                //     BinaryOp::GE  => Unit::Sge([r, a, b], comment_opt),
                //     BinaryOp::GT  => Unit::Sgt([r, a, b], comment_opt),
                //     BinaryOp::LT  => Unit::Slt([r, a, b], comment_opt),
                //     BinaryOp::LE  => Unit::Sle([r, a, b], comment_opt),
                //     BinaryOp::NE  => Unit::Sne([r, a, b], comment_opt),
                // };
                // units.push(unit);

                // (a_d + b_d + 1, Num::Reg(reg))
            }
            Expr::Ternary { cond, if_t, if_f } => {
                unreachable!();
            }
        }
    }

    fn translate_rv(
        &mut self,
        units: &mut Vec<mips::ast::Unit>,
        rv: myps::ast::Rv,
    ) -> (usize, mips::ast::Num) {
        use mips::ast::Num;
        use myps::ast::{Var, Rv};

        match rv {
            Rv::Expr(expr) => {
                unreachable!();
                self.translate_expr(units, None, expr, &mut None)
            },
            Rv::Dev(dev) => {
                unreachable!();
            },
            Rv::Var(Var { key, fixed }) => {
                unreachable!();
                // (0, self.get_?(&key))
            },
        }
    }
}

fn main() {
    let myps_path = std::env::args().skip(1).next().unwrap();
    let myps_src = std::fs::read_to_string(myps_path).unwrap();

    let program_item = myps::lexer::lex_str(&myps_src).unwrap();
    let translator = Translator::translate_myps(program_item);

    // println!("{:#?}", translator);

    let w = (translator.units.len() as f64 - 1.0)
        .log10()
        .floor()
        .max(0_f64) as usize
        + 1;
    for (i, unit) in translator.units.iter().enumerate() {
        println!("{:>w$}: {}", i, unit, w = w);
    }
}
