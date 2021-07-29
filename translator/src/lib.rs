#![feature(box_patterns)]
#![feature(bool_to_option)]
#![feature(stmt_expr_attributes)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
#![allow(unused_assignments)]
#![allow(dead_code)]
use std::collections::{BTreeMap, BTreeSet};
use std::convert::{TryFrom, TryInto};
use std::ops::Range;

use maplit::{btreemap, btreeset};

use mips::ast::{DevBase, FixMode, MipsNode, RegBase, RegLit};
use mips::{Alias, Aliases, Mips, MipsError, MipsResult};

#[derive(Clone, Debug)]
pub struct Translator {
    aliases: Aliases,
    next_index: usize,
}

impl Default for Translator {
    fn default() -> Self {
        let aliases = Aliases::default();
        Self {
            aliases,
            next_index: 0,
        }
    }
}

fn shift_scope(scope: &mut Range<usize>, offset: usize) {
    scope.start += offset;
    scope.end += offset;
}

fn shift_scopes(lines: &mut Vec<mips::ast::Line>, offset: usize) {
    for line in lines.iter_mut() {
        for arg in line.stmt.iter_args_mut() {
            if let Some(reg_lit) = arg.as_reg_lit_mut() {
                if let FixMode::Scoped(s, e) = &mut reg_lit.fix_mode {
                    *s += offset;
                    *e += offset;
                }
            }
        }
        // shift_scope(scope, offset);
    }
}

fn compare_scopes(lhs: &Range<usize>, rhs: &Range<usize>) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let lhs_size = lhs.end - lhs.start;
    let rhs_size = rhs.end - rhs.start;
    let size_ord = lhs_size.cmp(&rhs_size);
    if matches!(size_ord, Ordering::Equal) {
        lhs.start.cmp(&rhs.start)
    } else {
        size_ord
    }
}

impl Translator {
    pub fn next_reg_lit(&mut self, indirections: usize, fixed: bool) -> mips::ast::RegLit {
        let index = self.next_index;
        let fix_mode = fixed.into();
        let reg_lit = mips::ast::RegLit {
            index,
            indirections,
            fix_mode,
        };
        self.next_index += 1;
        reg_lit
    }

    pub fn next_reg_base(&mut self, indirections: usize, fixed: bool) -> mips::ast::RegBase {
        RegBase::Lit(self.next_reg_lit(indirections, fixed))
    }

    fn get_alias(&self, key: &String) -> Option<&Alias> {
        self.aliases.get(key)
    }

    fn try_get_alias(&self, key: &String) -> MipsResult<&Alias> {
        self.get_alias(key).ok_or(MipsError::alias_undefined(key))
    }

    fn has_reg(&self, key: &String) -> bool {
        self.get_alias(key)
            .map(|alias| matches!(alias, Alias::Reg(..)))
            .unwrap_or(false)
    }

    fn try_has_reg(&self, key: &String) -> MipsResult<bool> {
        let alias = self.try_get_alias(key).unwrap();
        Ok(matches!(alias, Alias::Reg(..)))
    }

    fn get_reg(&self, key: &String) -> Option<mips::ast::RegBase> {
        if let Some(Alias::Reg(reg_base)) = self.get_alias(key) {
            Some(reg_base.clone())
        } else {
            None
        }
    }

    fn try_get_reg(&self, key: &String) -> MipsResult<mips::ast::RegBase> {
        let alias = self.try_get_alias(key)?;
        if let Alias::Reg(reg_base) = alias {
            Ok(*reg_base)
        } else {
            Err(MipsError::alias_wrong_kind("a register", alias))
        }
    }

    fn try_get_reg_or_next(
        &mut self,
        key: &String,
        indirections: usize,
        fixed: bool,
    ) -> MipsResult<mips::ast::RegBase> {
        if let Some(Alias::Reg(reg_base)) = self.get_alias(key) {
            let mut reg_base = *reg_base;
            reg_base.set_fixed(reg_base.fixed() || fixed);
            Ok(reg_base)
        } else {
            let reg_base = self.next_reg_base(indirections, fixed);
            Ok(reg_base)
        }
    }

    fn try_has_dev(&self, key: &String) -> MipsResult<()> {
        let alias = self.try_get_alias(key)?;
        if matches!(alias, Alias::Dev(..)) {
            Ok(())
        } else {
            Err(MipsError::alias_wrong_kind("a device", alias))
        }
    }

    fn try_get_dev_base(&self, key: &String) -> MipsResult<mips::ast::DevBase> {
        let alias = self.try_get_alias(key)?;
        if let Alias::Dev(dev) = alias {
            Ok(dev.clone())
        } else {
            Err(MipsError::alias_wrong_kind("a device", alias))
        }
    }

    fn unwrap_reg_base(&mut self, reg_base_opt: Option<mips::ast::RegBase>) -> mips::ast::RegBase {
        use mips::ast::{Reg, RegBase};

        reg_base_opt.unwrap_or_else(|| {
            let reg_lit = self.next_reg_lit(0, false);
            RegBase::Lit(reg_lit)
        })
    }

    pub fn translate_item(&mut self, item: myps::ast::Item) -> MipsResult<Vec<mips::ast::Line>> {
        use mips::ast::{Arg, FixMode, Line, RegLit};
        use myps::ast::{Block, Item};

        // Collect lines from inner items
        let (mut lines, comment_opt) = match item {
            Item::Block(block, comment_opt) => {
                let lines = self.translate_block(block).unwrap();
                (lines, comment_opt)
            }
            Item::Stmt(stmt, comment_opt) => {
                let stmts = self.translate_stmt(stmt).unwrap();
                let lines = stmts
                    .into_iter()
                    .map(Line::new_no_comment)
                    .collect::<Vec<_>>();
                (lines, comment_opt)
            }
        };
        // Swap first comment
        if comment_opt.is_some() {
            if let Some(first) = lines.first_mut() {
                first.comment_opt = comment_opt;
            }
        }
        Ok(lines)
    }

    fn translate_items(&mut self, items: Vec<myps::ast::Item>) -> MipsResult<Vec<mips::ast::Line>> {
        use mips::ast::{Arg, FixMode, Line, Stmt};
        use myps::ast::{Block, Item};

        let mut lines = Vec::new();
        // let mut scopes = Vec::new();
        for item in items {
            let mut item_lines = self.translate_item(item).unwrap();
            for line in item_lines.iter_mut() {
                for arg in line.stmt.iter_args_mut() {
                    if let Some(reg_lit) = arg.as_reg_lit_mut() {
                        if let FixMode::Scoped(s, e) = &mut reg_lit.fix_mode {
                            *s += lines.len();
                            *e += lines.len();
                        }
                    }
                }
            }
            lines.extend(item_lines);
        }
        Ok(lines)
    }

    fn translate_block(&mut self, block: myps::ast::Block) -> MipsResult<Vec<mips::ast::Line>> {
        use mips::ast::{Arg, Line, LineAbs, LineRel, Num, Reg, Stmt};
        use myps::ast::{Branch, Var};
        use std::iter::once;

        fn transform_condition(
            translator: &mut Translator,
            mut cond_stmts: Vec<mips::ast::Stmt>,
            cond_num: mips::ast::Num,
        ) -> Vec<mips::ast::Stmt> {
            use mips::ast::Stmt;

            let c = Arg::Num(Num::Lit(0_f64));
            #[rustfmt::skip]
            if let Some(cond_stmt) = cond_stmts.pop() {
                // Having a statement on the stack means that the register of this last statement
                // is being thrown out, and so we need to decrement next_index.
                translator.next_index -= 1;
                let cond_stmt = match cond_stmt {
                    Stmt::Sap (..)        => unimplemented!(),
                    Stmt::Sapz(..)        => unimplemented!(),
                    Stmt::Sdns([_, d,  ]) => Stmt::Brdse([d,    c]),
                    Stmt::Sdse([_, d,  ]) => Stmt::Brdns([d,    c]),
                    Stmt::Seq ([_, a, b]) => Stmt::Brne ([a, b, c]),
                    Stmt::Seqz([_, a   ]) => Stmt::Brnez([a,    c]),
                    Stmt::Sge ([_, a, b]) => Stmt::Brlt ([a, b, c]),
                    Stmt::Sgez([_, a   ]) => Stmt::Brltz([a,    c]),
                    Stmt::Sgt ([_, a, b]) => Stmt::Brle ([a, b, c]),
                    Stmt::Sgtz([_, a   ]) => Stmt::Brlez([a,    c]),
                    Stmt::Sle ([_, a, b]) => Stmt::Brgt ([a, b, c]),
                    Stmt::Slez([_, a   ]) => Stmt::Brgtz([a,    c]),
                    Stmt::Slt ([_, a, b]) => Stmt::Brge ([a, b, c]),
                    Stmt::Sltz([_, a   ]) => Stmt::Brgez([a,    c]),
                    Stmt::Sna (..)        => unimplemented!(),
                    Stmt::Snaz(..)        => unimplemented!(),
                    Stmt::Sne ([_, a, b]) => Stmt::Breq ([a, b, c]),
                    Stmt::Snez([_, a])    => Stmt::Breqz([a,    c]),
                    _ => {
                        cond_stmts.push(cond_stmt);
                        Stmt::Brnez([cond_num.into(), c])
                    },
                };
                cond_stmts.push(cond_stmt);
            } else {
                let cond_stmt = Stmt::Brnez([cond_num.into(), c]);
                cond_stmts.push(cond_stmt);
            }
            cond_stmts
        }

        fn update_branch(cond_stmts: &mut Vec<mips::ast::Stmt>, c: mips::ast::Arg) {
            *cond_stmts
                .last_mut()
                .unwrap()
                .args_mut()
                .last_mut()
                .unwrap() = c;
        }

        fn tag_string(i: usize) -> String {
            format!("endChain{}", i)
        }

        fn tag(i: usize) -> Arg {
            Arg::LineAbs(LineAbs(tag_string(i).into()))
        }

        let myps::ast::Block { branch, items } = block;
        let mut lines = match branch {
            Branch::Program => self.translate_items(items).unwrap(),
            Branch::Loop => {
                let mut lines = self.translate_items(items).unwrap();
                let line_rel = LineRel((-(lines.len() as f64)).into());
                let stmt_jr = Stmt::Jr([line_rel.into()]);
                lines.push(Line::new_no_comment(stmt_jr));
                lines
            }
            Branch::If { cond, chain_id_opt } => {
                // Translate condition expression to statements
                let (cond_num, cond_stmts) = self.translate_expr(None, cond).unwrap();
                // Transform last condition statement to branch
                let mut cond_stmts = transform_condition(self, cond_stmts, cond_num);
                // Translate body items to lines
                let mut body_lines = self.translate_items(items).unwrap();
                // Shift body hints
                shift_scopes(&mut body_lines, cond_stmts.len());
                // If part of chain, add jump to post if-elif-else chain tag
                if let Some(chain_id) = chain_id_opt {
                    let tag = tag(chain_id).into();
                    body_lines.push(Line::new_no_comment(Stmt::J([tag])));
                }
                // Update condition branch jump
                let jump_by = (cond_stmts.len() + body_lines.len()) as i64;
                let jump_f = Arg::LineRel(jump_by.into());
                update_branch(&mut cond_stmts, jump_f);
                let cond_lines = cond_stmts
                    .into_iter()
                    .map(Line::new_no_comment)
                    .collect::<Vec<_>>();
                // Construct lines
                let lines = cond_lines
                    .into_iter()
                    .chain(body_lines.into_iter())
                    .collect::<Vec<_>>();
                lines
            }
            Branch::Elif {
                cond,
                chain_id,
                end_chain,
            } => {
                // Translate condition expression to statements
                let (cond_num, cond_stmts) = self.translate_expr(None, cond).unwrap();
                // Transform last condition statement to branch
                let mut cond_stmts = transform_condition(self, cond_stmts, cond_num);
                // Translate body items to lines
                let mut body_lines = self.translate_items(items).unwrap();
                // Shift body hints
                shift_scopes(&mut body_lines, cond_stmts.len());
                // Push if-elif-else chain line
                // let tag = tag(chain_id).into();
                let chain_stmt = if end_chain {
                    // If end of chain, add post if-elif-else chain tag
                    Stmt::Tag([tag_string(chain_id).into()])
                } else {
                    // Else not end of chain, add jump to post if-elif-else chain tag
                    Stmt::J([tag(chain_id)])
                };
                body_lines.push(Line::new_no_comment(chain_stmt));
                // Update condition branch jump
                let jump_f = Arg::LineRel((body_lines.len() as i64 + 1).into());
                update_branch(&mut cond_stmts, jump_f);
                let cond_lines = cond_stmts
                    .into_iter()
                    .map(Line::new_no_comment)
                    .collect::<Vec<_>>();
                // Construct lines
                let lines = cond_lines
                    .into_iter()
                    .chain(body_lines.into_iter())
                    .collect::<Vec<_>>();
                lines
            }
            Branch::Else { chain_id } => {
                // Translate body items to lines
                let mut lines = self.translate_items(items).unwrap();
                // Push post if-elif-else chain tag
                let chain_stmt = Stmt::Tag([tag(chain_id).into()]);
                lines.push(Line::new_no_comment(chain_stmt));
                lines
            }
            Branch::While { cond } => {
                // Translate condition expression to statements
                let (cond_num, cond_stmts) = self.translate_expr(None, cond).unwrap();
                // Transform last condition statement to branch
                let mut cond_stmts = transform_condition(self, cond_stmts, cond_num);
                // Translate body items to lines
                let mut body_lines = self.translate_items(items).unwrap();
                // Shift body hints
                shift_scopes(&mut body_lines, cond_stmts.len());
                // Push backwards jump
                body_lines.push({
                    let jump_by = -((body_lines.len() + cond_stmts.len()) as i64);
                    let jump_back = Arg::LineRel(jump_by.into());
                    let stmt = Stmt::Jr([jump_back]);
                    Line::new_no_comment(stmt)
                });
                // Update condition branch jump
                let jump_f = Arg::LineRel((body_lines.len() as i64 + 1).into());
                update_branch(&mut cond_stmts, jump_f);
                let cond_lines = cond_stmts
                    .into_iter()
                    .map(Line::new_no_comment)
                    .collect::<Vec<_>>();
                // Construct lines
                let lines = cond_lines
                    .into_iter()
                    .chain(body_lines.into_iter())
                    .collect();
                lines
            }
            Branch::For(Var { key, .. }, s, e, step) => {
                // Get register for loop index (i)
                let i_reg_lit = self.next_reg_lit(0, true);
                let i_index = i_reg_lit.index;
                let i_reg_base = RegBase::Lit(i_reg_lit);
                let alias = Alias::Reg(i_reg_base);
                self.aliases.insert(key, alias);
                // Translate loop index initialization to statements
                let i_lines = {
                    // Translate index start expression to statements
                    let (s_num, mut s_stmts) = self.translate_expr(Some(i_reg_base), s).unwrap();
                    // If start expression is a literal number then add a Move statement
                    if matches!(s_num, Num::Lit(_)) {
                        let lhs = Arg::Reg(i_reg_base.into());
                        let rhs = Arg::Num(s_num);
                        s_stmts.push(Stmt::Move([lhs, rhs]));
                    }
                    s_stmts
                        .into_iter()
                        .map(Line::new_no_comment)
                        .collect::<Vec<_>>()
                };
                // Translate loop index end expression to statements
                let (e_num, e_lines) = {
                    let (e_num, e_stmts) = self.translate_expr(None, e).unwrap();
                    let e_lines = e_stmts
                        .into_iter()
                        .map(Line::new_no_comment)
                        .collect::<Vec<_>>();
                    (e_num, e_lines)
                };
                // Translate step expression to statements
                let (step_num, step_lines) = {
                    let (step_num, step_stmts) = self.translate_expr(None, step).unwrap();
                    let step_lines = step_stmts
                        .into_iter()
                        .map(Line::new_no_comment)
                        .collect::<Vec<_>>();
                    (step_num, step_lines)
                };
                // Skip body if index less than end value (brlt)
                let mut cond_stmts = {
                    let a = Arg::Num(Num::Reg(i_reg_base));
                    let b = Arg::Num(e_num);
                    let c = Arg::LineRel(0.into());
                    vec![Stmt::Brlt([a, b, c])]
                };
                // Translate body items to lines
                let mut body_lines = self.translate_items(items).unwrap();
                // Shift body hints
                let shift_by =
                    i_lines.len() + e_lines.len() + step_lines.len() + cond_stmts.len() + 1;
                shift_scopes(&mut body_lines, shift_by);
                // Push increment statement
                body_lines.push({
                    let r = Arg::Reg(Reg::Base(i_reg_base));
                    let a = Arg::Num(Num::Reg(i_reg_base));
                    let b = Arg::Num(step_num);
                    let stmt = Stmt::Add([r, a, b]);
                    Line::new_no_comment(stmt)
                });
                // Push backwards jump
                body_lines.push({
                    let jump_by = -((body_lines.len() + cond_stmts.len()) as i64);
                    let jump_back = Arg::LineRel(jump_by.into());
                    let stmt = Stmt::Jr([jump_back]);
                    Line::new_no_comment(stmt)
                });
                // Update condition branch jump
                let jump_by = body_lines.len() as i64 + 1;
                let jump_forward = Arg::LineRel(jump_by.into());
                update_branch(&mut cond_stmts, jump_forward);
                let cond_lines = cond_stmts
                    .into_iter()
                    .map(Line::new_no_comment)
                    .collect::<Vec<_>>();
                // Collect lines
                let lines = i_lines
                    .into_iter()
                    .chain(e_lines.into_iter())
                    .chain(step_lines.into_iter())
                    .chain(cond_lines.into_iter())
                    .chain(body_lines.into_iter())
                    .collect::<Vec<_>>();
                lines
            }
            Branch::Tag(tag) => {
                let tag_stmt = Stmt::Tag([Arg::String(tag)]);
                let mut body_lines = self.translate_items(items).unwrap();
                shift_scopes(&mut body_lines, 1);
                let lines = once(tag_stmt)
                    .map(Line::new_no_comment)
                    .chain(body_lines.into_iter())
                    .collect();
                lines
            }
        };
        // Swap Fixed to Scoped
        let (s, e) = (0, lines.len());
        for line in lines.iter_mut() {
            for arg in line.stmt.iter_args_mut() {
                if let Some(reg_lit) = arg.as_reg_lit_mut() {
                    if matches!(reg_lit.fix_mode, FixMode::Fixed) {
                        let scope = FixMode::Scoped(s, e);
                        reg_lit.fix_mode = scope;
                    }
                }
            }
        }
        Ok(lines)
    }

    fn translate_stmt(&mut self, stmt: myps::ast::Stmt) -> MipsResult<Vec<mips::ast::Stmt>> {
        use mips::ast::{Arg, Stmt};
        use myps::ast::{Lv, Var};
        use std::iter::once;

        match stmt {
            myps::ast::Stmt::Fix(names) => {
                let stmts = names
                    .into_iter()
                    .map(|name| {
                        let reg_base = self.next_reg_base(0, true);
                        let alias = Alias::Reg(reg_base);
                        self.aliases.insert(name.clone(), alias);
                        Stmt::Alias([name.into(), reg_base.into()])
                    })
                    .collect();
                Ok(stmts)
            }
            myps::ast::Stmt::Asn(lv, rv) => {
                let (stmts, alias_pair) = self.translate_assignment(lv, rv).unwrap();
                if let Some((key, alias)) = alias_pair {
                    self.aliases.insert(key, alias);
                }
                Ok(stmts)
            }
            myps::ast::Stmt::SelfAsn {
                op,
                lhs: Var { key, .. },
                rhs,
            } => {
                use myps::ast::BinaryOp;

                let reg_base = self.try_get_reg(&key).unwrap();
                let r = reg_base.into();
                let a = reg_base.into();
                let (b_num, b_stmts) = self.translate_expr(None, rhs).unwrap();
                let b = b_num.into();
                let stmt = match op {
                    BinaryOp::Add => Stmt::Add([r, a, b]),
                    BinaryOp::Sub => Stmt::Sub([r, a, b]),
                    BinaryOp::Mul => Stmt::Mul([r, a, b]),
                    BinaryOp::Div => Stmt::Div([r, a, b]),
                    BinaryOp::Rem => Stmt::Mod([r, a, b]),
                    _ => unreachable!(),
                };
                let stmts = b_stmts.into_iter().chain(once(stmt)).collect();
                Ok(stmts)
            }
            myps::ast::Stmt::Mips(mut stmt) => {
                for arg in stmt.iter_args_mut() {
                    match arg {
                        Arg::Dev(..) | Arg::Reg(..) | Arg::Num(..) => {
                            *arg = arg.clone().reduce(&self.aliases).unwrap();
                        }
                        _ => {}
                    }
                }
                Ok(vec![stmt])
            }
            myps::ast::Stmt::Empty => Ok(vec![Stmt::Empty([])]),
        }
    }

    fn translate_assignment(
        &mut self,
        lhs: myps::ast::Lv,
        rhs: myps::ast::Rv,
    ) -> MipsResult<(Vec<mips::ast::Stmt>, Option<(String, Alias)>)> {
        use mips::ast::{Arg, Dev, Num, Reg, Stmt};
        use myps::ast::{Lv, Rv, Var};

        fn translate_s(
            translator: &mut Translator,
            num: mips::ast::Num,
            dev: myps::ast::Dev,
            param: String,
        ) -> MipsResult<Vec<Stmt>> {
            let (dev, mut stmts) = translator.translate_dev(dev).unwrap();
            let d = dev.into();
            let p = param.into();
            let r = num.into();
            let stmt = Stmt::S([d, p, r]);
            stmts.push(stmt);
            Ok(stmts)
        }

        fn translate_sb(
            translator: &mut Translator,
            num: mips::ast::Num,
            hash: myps::ast::Num,
            param: String,
        ) -> MipsResult<Vec<Stmt>> {
            let (hash, mut stmts) = translator.translate_num(None, hash).unwrap();
            let h = hash.into();
            let p = param.into();
            let r = num.into();
            let stmt = Stmt::Sb([h, p, r]);
            stmts.push(stmt);
            Ok(stmts)
        }

        match rhs {
            Rv::Expr(expr) => match lhs {
                Lv::DevParam { dev, param } => {
                    let (num, mut stmts) = self.translate_expr(None, expr).unwrap();
                    stmts.extend(translate_s(self, num, dev, param).unwrap());
                    Ok((stmts, None))
                }
                Lv::NetParam { hash, param } => {
                    let (num, mut stmts) = self.translate_expr(None, expr).unwrap();
                    stmts.extend(translate_sb(self, num, hash, param).unwrap());
                    Ok((stmts, None))
                }
                Lv::Var(Var {
                    key: lv_key,
                    fixed: lv_fixed,
                }) => {
                    let lv_reg_base = self.try_get_reg_or_next(&lv_key, 0, lv_fixed).unwrap();
                    let fixed = lv_fixed || lv_reg_base.fixed();
                    let (num, mut stmts) = self.translate_expr(Some(lv_reg_base), expr).unwrap();
                    if stmts.is_empty() {
                        let r = lv_reg_base.into();
                        let a = num.into();
                        stmts.push(Stmt::Move([r, a]));
                    }
                    let alias = Alias::Reg(lv_reg_base);
                    let alias_pair = (lv_key, alias);
                    Ok((stmts, Some(alias_pair)))
                }
                Lv::Def(lv_key) => {
                    let (num, _) = self.translate_expr(None, expr).unwrap();
                    match num {
                        Num::Lit(n) => {
                            let alias = Alias::Num(n);
                            let alias_pair = (lv_key, alias);
                            Ok((Vec::new(), Some(alias_pair)))
                        }
                        Num::Alias(rv_key) => {
                            let rv_alias = self.try_get_alias(&rv_key).unwrap();
                            unimplemented!();
                        }
                        Num::Reg(_) => unreachable!(),
                    }
                }
            },
            Rv::Var(Var {
                key: rv_key,
                fixed: rv_fixed,
            }) => {
                let rv_alias = self.try_get_alias(&rv_key).unwrap().clone();
                match lhs {
                    Lv::DevParam { dev, param } => {
                        let num = Num::try_from(&rv_alias).unwrap();
                        let stmts = translate_s(self, num, dev, param).unwrap();
                        Ok((stmts, None))
                    }
                    Lv::NetParam { hash, param } => {
                        let num = Num::try_from(&rv_alias).unwrap();
                        let stmts = translate_sb(self, num, hash, param).unwrap();
                        Ok((stmts, None))
                    }
                    Lv::Var(Var {
                        key: lv_key,
                        fixed: lv_fixed,
                    }) => match rv_alias {
                        Alias::Num(n) => {
                            let lv_reg_base =
                                self.try_get_reg_or_next(&lv_key, 0, lv_fixed).unwrap();
                            let stmt =
                                Stmt::Move([Arg::Reg(lv_reg_base.into()), Arg::Num(n.into())]);
                            let stmts = vec![stmt];
                            let alias = lv_reg_base.into();
                            let alias_pair = (lv_key, alias);
                            Ok((stmts, Some(alias_pair)))
                        }
                        Alias::Dev(..) => {
                            unimplemented!();
                        }
                        Alias::Reg(rv_reg_base) => {
                            let lv_reg_base =
                                self.try_get_reg_or_next(&lv_key, 0, lv_fixed).unwrap();
                            let stmt = Stmt::Move([
                                Arg::Reg(lv_reg_base.into()),
                                Arg::Num(rv_reg_base.into()),
                            ]);
                            let stmts = vec![stmt];
                            let alias = lv_reg_base.into();
                            let alias_pair = (lv_key, alias);
                            Ok((stmts, Some(alias_pair)))
                        }
                    },
                    Lv::Def(_) => {
                        unreachable!();
                    }
                }
            }
            Rv::Dev(dev) => {
                let (dev_base, stmts) = self.translate_dev(dev).unwrap();
                if let Lv::Var(Var { key: lv_key, .. }) = lhs {
                    let stmts = vec![Stmt::Alias([lv_key.clone().into(), dev_base.into()])];
                    let alias_pair = (lv_key, dev_base.into());
                    Ok((stmts, Some(alias_pair)))
                } else {
                    unreachable!();
                }
            }
        }
    }

    fn translate_expr(
        &mut self,
        reg_base_opt: Option<mips::ast::RegBase>,
        expr: myps::ast::Expr,
    ) -> MipsResult<(mips::ast::Num, Vec<mips::ast::Stmt>)> {
        use mips::ast::{Arg, Num, Reg, Stmt};
        use myps::ast::Expr;
        use std::iter::once;

        match expr {
            Expr::Num(num) => self.translate_num(reg_base_opt, num),
            Expr::Unary { op, rhs } => {
                use myps::ast::UnaryOp;

                let reg_base = self.unwrap_reg_base(reg_base_opt);
                let (b_num, b_stmts) = self.translate_num(None, rhs).unwrap();
                let r = reg_base.into();
                let b = b_num.into();
                let op_stmt = match op {
                    UnaryOp::Inv => {
                        let a = Num::Lit(0_f64).into();
                        Stmt::Sub([r, a, b])
                    }
                    UnaryOp::Not => {
                        let a = b.clone();
                        Stmt::Nor([r, a, b])
                    }
                };
                let stmts = b_stmts.into_iter().chain(once(op_stmt)).collect();
                Ok((reg_base.into(), stmts))
            }
            Expr::Binary { op, lhs, rhs } => {
                use myps::ast::BinaryOp;

                let reg_base = self.unwrap_reg_base(reg_base_opt);
                let (a_num, a_stmts) = self.translate_num(None, lhs).unwrap();
                let (b_num, b_stmts) = self.translate_num(None, rhs).unwrap();
                let r = reg_base.into();
                let a = a_num.into();
                let b = b_num.into();
                #[rustfmt::skip]
                let op_stmts = match op {
                    // Numerical
                    BinaryOp::Add => vec![Stmt::Add([r, a, b])],
                    BinaryOp::Sub => vec![Stmt::Sub([r, a, b])],
                    BinaryOp::Mul => vec![Stmt::Mul([r, a, b])],
                    BinaryOp::Div => vec![Stmt::Div([r, a, b])],
                    BinaryOp::Rem => vec![Stmt::Mod([r, a, b])],
                    BinaryOp::Pow => {
                        // log_b x = (log_a x)/(log_a b)
                        vec![
                            Stmt::Log([r.clone(), a.clone()]),
                            Stmt::Mul([r.clone(), b, r.clone()]),
                            Stmt::Exp([r.clone(), r]),
                        ]
                    },
                    // Logical
                    BinaryOp::And => vec![Stmt::And([r, a, b])],
                    BinaryOp::Nor => unreachable!("NOR {:?} {:?} {:?}", r, a, b),
                    BinaryOp::Or  => vec![Stmt::Or ([r, a, b])],
                    BinaryOp::Xor => vec![Stmt::Xor([r, a, b])],
                    // Relational
                    BinaryOp::Eq  => vec![Stmt::Seq([r, a, b])],
                    BinaryOp::Ge  => vec![Stmt::Sge([r, a, b])],
                    BinaryOp::Gt  => vec![Stmt::Sgt([r, a, b])],
                    BinaryOp::Lt  => vec![Stmt::Slt([r, a, b])],
                    BinaryOp::Le  => vec![Stmt::Sle([r, a, b])],
                    BinaryOp::Ne  => vec![Stmt::Sne([r, a, b])],
                };
                let stmts = a_stmts
                    .into_iter()
                    .chain(b_stmts.into_iter())
                    .chain(op_stmts.into_iter())
                    .collect();
                Ok((reg_base.into(), stmts))
            }
            Expr::Ternary { cond, if_t, if_f } => {
                let reg_base = self.unwrap_reg_base(reg_base_opt);
                let (a, a_stmts) = self.translate_num(None, cond).unwrap();
                let (b, b_stmts) = self.translate_num(None, if_t).unwrap();
                let (c, c_stmts) = self.translate_num(None, if_f).unwrap();
                let stmt = Stmt::Select([reg_base.into(), a.into(), b.into(), c.into()]);
                let stmts = a_stmts
                    .into_iter()
                    .chain(b_stmts.into_iter())
                    .chain(c_stmts.into_iter())
                    .chain(once(stmt))
                    .collect();
                Ok((reg_base.into(), stmts))
            }
        }
    }

    fn translate_num(
        &mut self,
        reg_base_opt: Option<mips::ast::RegBase>,
        num: myps::ast::Num,
    ) -> MipsResult<(mips::ast::Num, Vec<mips::ast::Stmt>)> {
        use mips::ast::{Arg, Dev, Num, Reg, Stmt};
        use myps::ast::Var;
        use std::iter::once;

        match num {
            myps::ast::Num::Lit(n) => Ok((mips::ast::Num::Lit(n), Vec::new())),
            myps::ast::Num::Var(Var { key, .. }) => {
                let alias = self.try_get_alias(&key).unwrap();
                let num = alias.try_into().unwrap();
                Ok((num, Vec::new()))
            }
            myps::ast::Num::Expr(box expr) => self.translate_expr(reg_base_opt, expr),
            myps::ast::Num::Func(box func) => self.translate_func(reg_base_opt, func),
            myps::ast::Num::DevParam { dev, param } => {
                let reg_base = self.unwrap_reg_base(reg_base_opt);
                let (dev_base, dev_stmts) = self.translate_dev(dev).unwrap();
                let l_stmt = Stmt::L([reg_base.into(), dev_base.into(), param.into()]);
                let stmts = dev_stmts.into_iter().chain(once(l_stmt)).collect();
                Ok((reg_base.into(), stmts))
            }
            myps::ast::Num::DevSlot {
                dev,
                box slot,
                param,
            } => {
                let reg_base = self.unwrap_reg_base(reg_base_opt);
                let (dev_base, dev_stmts) = self.translate_dev(dev).unwrap();
                let (slot, slot_stmts) = self.translate_num(None, slot).unwrap();
                let ls_stmt =
                    Stmt::Ls([
                        reg_base.into(),
                        dev_base.into(),
                        slot.into(),
                        param.into()
                    ]);
                let stmts = dev_stmts
                    .into_iter()
                    .chain(slot_stmts.into_iter())
                    .chain(once(ls_stmt))
                    .collect();
                Ok((reg_base.into(), stmts))
            }
            myps::ast::Num::DevReagent {
                dev,
                box mode,
                reagent,
            } => {
                let reg_base = self.unwrap_reg_base(reg_base_opt);
                let (dev_base, dev_stmts) = self.translate_dev(dev).unwrap();
                let (mode, mode_stmts) = self.translate_num(None, mode).unwrap();
                let lr_stmt = Stmt::Lr([
                    reg_base.into(),
                    dev_base.into(),
                    mode.into(),
                    reagent.into(),
                ]);
                let stmts = dev_stmts
                    .into_iter()
                    .chain(mode_stmts.into_iter())
                    .chain(once(lr_stmt))
                    .collect();
                Ok((reg_base.into(), stmts))
            }
            myps::ast::Num::NetParam {
                box hash,
                box mode,
                param,
            } => {
                let reg_base = self.unwrap_reg_base(reg_base_opt);
                let (hash, hash_stmts) = self.translate_num(None, hash).unwrap();
                let mode = if let myps::ast::Num::Var(mut var) = mode {
                    var.key = var.key.to_lowercase();
                    myps::ast::Num::Var(var)
                } else {
                    mode
                };
                let (mode, mode_stmts) = self.translate_num(None, mode).unwrap();
                let lb_stmt = Stmt::Lb([
                    reg_base.into(),
                    hash.into(),
                    param.into(),
                    mode.into()
                ]);
                let stmts = hash_stmts
                    .into_iter()
                    .chain(mode_stmts.into_iter())
                    .chain(once(lb_stmt))
                    .collect();
                Ok((reg_base.into(), stmts))
            }
        }
    }

    fn translate_func(
        &mut self,
        reg_base_opt: Option<mips::ast::RegBase>,
        func: myps::ast::Func,
    ) -> MipsResult<(mips::ast::Num, Vec<mips::ast::Stmt>)> {
        use mips::ast::{Arg, Dev, Num, Reg, Stmt};
        use myps::ast::{Func, Var};
        use std::iter::once;

        macro_rules! translate_fun {
            ($self:ident, $reg_opt:ident, [], $mips:path) => {{
                let reg_base = self.unwrap_reg_base($reg_opt);
                let stmt = $mips([reg_base.into()]);
                Ok((reg_base.into(), vec![stmt]))
            }};
            ($self:ident, $reg_opt:ident, [$a:ident], $mips:path) => {{
                let reg_base = self.unwrap_reg_base($reg_opt);
                let (a, a_stmts) = $self.translate_arg($a).unwrap();
                let stmt = $mips([reg_base.into(), a.into()]);
                let stmts = a_stmts.into_iter().chain(once(stmt)).collect();
                Ok((reg_base.into(), stmts))
            }};
            ($self:ident, $reg_opt:ident, [$a:ident, $b:ident], $mips:path) => {{
                let reg_base = self.unwrap_reg_base($reg_opt);
                let (a, a_stmts) = $self.translate_arg($a).unwrap();
                let (b, b_stmts) = $self.translate_arg($b).unwrap();
                let stmt = $mips([reg_base.into(), a.into(), b.into()]);
                let stmts = a_stmts
                    .into_iter()
                    .chain(b_stmts.into_iter())
                    .chain(once(stmt))
                    .collect();
                Ok((reg_base.into(), stmts))
            }};
        }

        #[rustfmt::skip]
        match func {
            Func::Dns  ([d   ]) => translate_fun!(self, reg_base_opt, [d   ], Stmt::Sdns),
            Func::Dse  ([d   ]) => translate_fun!(self, reg_base_opt, [d   ], Stmt::Sdse),
            Func::Abs  ([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Abs),
            Func::Acos ([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Acos),
            Func::Asin ([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Asin),
            Func::Ceil ([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Ceil),
            Func::Cos  ([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Cos),
            Func::Exp  ([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Exp),
            Func::Floor([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Floor),
            Func::Log  ([a, b]) => {
                let reg_base = self.unwrap_reg_base(reg_base_opt);
                let r = Arg::Reg(Reg::Base(reg_base));
                let (a, a_stmts) = self.translate_arg(a).unwrap();
                let (b, b_stmts) = self.translate_arg(b).unwrap();
                let log_stmts = vec![
                    Stmt::Log([r.clone(), a]),
                    Stmt::Log([r.clone(), b]),
                    Stmt::Div([r.clone(), r.clone(), r.clone()]),
                ];
                let stmts = a_stmts
                    .into_iter()
                    .chain(b_stmts.into_iter())
                    .chain(log_stmts.into_iter())
                    .collect();
                Ok((reg_base.into(), stmts))
            },
            Func::Ln   ([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Log),
            Func::Max  ([a, b]) => translate_fun!(self, reg_base_opt, [a, b], Stmt::Max),
            Func::Min  ([a, b]) => translate_fun!(self, reg_base_opt, [a, b], Stmt::Min),
            Func::Rand ([    ]) => translate_fun!(self, reg_base_opt, [    ], Stmt::Rand),
            Func::Round([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Round),
            Func::Sin  ([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Sin),
            Func::Sqrt ([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Sqrt),
            Func::Tan  ([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Tan),
            Func::Trunc([a   ]) => translate_fun!(self, reg_base_opt, [a   ], Stmt::Trunc),
            Func::Peek ([    ]) => translate_fun!(self, reg_base_opt, [    ], Stmt::Peek),
            Func::Pop  ([    ]) => translate_fun!(self, reg_base_opt, [    ], Stmt::Pop),
        }
    }

    fn translate_arg(
        &mut self,
        arg: myps::ast::Arg,
    ) -> MipsResult<(mips::ast::Arg, Vec<mips::ast::Stmt>)> {
        use mips::ast::Arg;

        match arg {
            myps::ast::Arg::Dev(dev) => {
                let (dev_base, stmts) = self.translate_dev(dev).unwrap();
                Ok((dev_base.into(), stmts))
            }
            myps::ast::Arg::Expr(expr) => {
                let (num, stmts) = self.translate_expr(None, expr).unwrap();
                Ok((num.into(), stmts))
            }
        }
    }

    fn translate_dev(
        &mut self,
        dev: myps::ast::Dev,
    ) -> MipsResult<(mips::ast::DevBase, Vec<mips::ast::Stmt>)> {
        use mips::ast::{Dev, DevBase, DevLit, Num, Reg, RegLit};
        use myps::ast::Var;

        match dev {
            myps::ast::Dev::Lit(i) => {
                let dev_lit = DevLit::new(i, 0);
                let dev_base = DevBase::Lit(dev_lit);
                Ok((dev_base, Vec::new()))
            }
            myps::ast::Dev::Expr(box expr) => {
                let (index_num, index_stmts) = self.translate_expr(None, expr).unwrap();
                match index_num {
                    Num::Lit(i) => {
                        let index = (i >= 0_f64)
                            .then_some(i as usize)
                            .ok_or(MipsError::index_invalid(i))?;
                        let dev_lit = DevLit {
                            index,
                            indirections: 0,
                        };
                        Ok((dev_lit.into(), index_stmts))
                    }
                    Num::Reg(..) | Num::Alias(..) => {
                        let index = match index_num {
                            Num::Reg(reg_base) => reg_base.index(),
                            Num::Alias(key) => self.try_get_reg(&key).unwrap().index(),
                            Num::Lit(..) => unreachable!(),
                        };
                        let dev_lit = DevLit {
                            index,
                            indirections: 1,
                        };
                        Ok((dev_lit.into(), index_stmts))
                    }
                }
            }
            myps::ast::Dev::Var(Var { key, .. }) => {
                let dev_base = self.try_get_dev_base(&key).unwrap();
                Ok((dev_base, Vec::new()))
            }
            myps::ast::Dev::DB => {
                let dev_base = DevBase::DB;
                Ok((dev_base, Vec::new()))
            }
        }
    }
}

