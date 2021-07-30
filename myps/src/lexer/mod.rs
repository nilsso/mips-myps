use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader};

use maplit::btreemap;
use pest::Parser;

use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};
// use mips::Alias;

use crate::ast::*;
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

const INDENT_SIZE: usize = 4;

pub fn lex_string(source: String) -> MypsResult<Item> {
    lex_lines(source.lines().map(str::to_owned))
}

pub fn lex_file<P: Into<PathBuf> + std::fmt::Debug>(path: P) -> MypsResult<Item> {
    let f = File::open(path.into()).unwrap();
    let f = BufReader::new(f);
    let lines = f.lines().collect::<Result<Vec<_>, _>>().unwrap();
    lex_lines(lines.into_iter())
}

pub fn lex_lines<'a>(line_iter: impl Iterator<Item = String>) -> MypsResult<Item> {
    let mut block_stack: Vec<(Block, Option<String>)> = vec![(Block::new(Branch::Program), None)];
    let mut indent_stack = vec![0_usize];
    let mut curr_indent = 0_usize;
    let mut expect_indent = false;

    for (i, line_src) in line_iter.enumerate() {
        let pair = MypsParser::parse(Rule::single_line, &line_src)
            .unwrap()
            .only_rule(Rule::single_line, "a line")
            .unwrap();
        let (spaces, item) = Item::try_from_pair(pair).unwrap();

        // Handle indent
        if !matches!(item, Item::Stmt(Stmt::Empty, _)) {
            let indent = if spaces % INDENT_SIZE != 0 {
                panic!("Invalid indent size {} (line {})", spaces, i);
            } else {
                spaces / INDENT_SIZE
            };
            if expect_indent {
                if indent <= curr_indent {
                    panic!(
                        "Expected indent (line {}, {}, {}, {:?})",
                        i, indent, curr_indent, indent_stack
                    );
                } else {
                    indent_stack.push(indent);
                    curr_indent = indent;
                    expect_indent = false;
                }
            } else {
                if indent < *indent_stack.last().unwrap() {
                    // Remove empties from previous head
                    let empties = {
                        let (head, _) = block_stack.last_mut().unwrap();
                        let i = 1 + head
                            .items
                            .iter()
                            .rposition(|item| !matches!(item, Item::Stmt(Stmt::Empty, _)))
                            .unwrap_or(head.items.len());
                        head.items.split_off(i)
                    };
                    // Pop indents and blocks until at new head
                    while indent < *indent_stack.last().unwrap() {
                        indent_stack.pop();
                        let (block, comment_opt) = block_stack.pop().unwrap();
                        let (head, _) = block_stack.last_mut().unwrap();
                        head.items.push(Item::Block(block, comment_opt));
                        curr_indent = *indent_stack.last().unwrap();
                    }
                    // Added removed empties to new head
                    let (head, _) = block_stack.last_mut().unwrap();
                    head.items.extend(empties);
                }
            }
        }
        // Place item
        match item {
            Item::Block(block, comment_opt) => {
                block_stack.push((block, comment_opt));
                expect_indent = true;
            }
            Item::Stmt(..) => {
                if !matches!(item, Item::Stmt(Stmt::Empty, _)) {
                    expect_indent = false;
                }
                match item {
                    _ => {}
                }
                let (head, _) = block_stack.last_mut().unwrap();
                head.items.push(item);
            }
        }
    }
    if expect_indent {
        panic!("Expected indent");
    }
    while block_stack.len() > 1 {
        // indent_stack.pop();
        let (block, comment_opt) = block_stack.pop().unwrap();
        let (head, _) = block_stack.last_mut().unwrap();
        head.items.push(Item::Block(block, comment_opt));
    }
    let (program_block, _) = block_stack.pop().unwrap();
    let mut program_item = Item::Block(program_block, None);

    // Validate/update if-elif-else chains
    fn validate_chains(item: &mut Item, next_chain_id: &mut usize) -> MypsResult<()> {
        if let Item::Block(block, ..) = item {
            let items = &mut block.items;
            for i in 0..items.len() {

                if matches!(items[i], Item::Block(..)) {
                    if items[i].is_if_elif_else() {
                        let j = i + 1;
                        let prev_chain_id = if i > 0 {
                            items[i - 1].chain_id()
                        } else {
                            None
                        };
                        let next_is_elif_else = j < items.len() && items[j].is_elif_else();
                        match &mut items[i] {
                            Item::Block(
                                Block {
                                    branch: Branch::If { chain_id_opt, .. },
                                    ..
                                },
                                ..,
                            ) => {
                                if next_is_elif_else {
                                    *chain_id_opt = Some(*next_chain_id);
                                    *next_chain_id += 1;
                                }
                            }
                            Item::Block(
                                Block {
                                    branch:
                                        Branch::Elif {
                                            chain_id,
                                            end_chain,
                                            ..
                                        },
                                    ..
                                },
                                ..,
                            ) => {
                                *chain_id = prev_chain_id.unwrap();
                                if next_is_elif_else {
                                    *end_chain = false;
                                }
                            }
                            Item::Block(
                                Block {
                                    branch: Branch::Else { chain_id, .. },
                                    ..
                                },
                                ..,
                            ) => {
                                *chain_id = prev_chain_id.unwrap();
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        let j = i + 1;
                        if j < items.len() && items[i + 1].is_elif_else() {
                            panic!();
                        }
                    }
                    validate_chains(&mut items[i], next_chain_id).unwrap();
                }
            }
        }
        Ok(())
    }

    let mut next_chain_id = 0_usize;
    validate_chains(&mut program_item, &mut next_chain_id).unwrap();

    let mut fixed_map = BTreeMap::<String, bool>::new();

    // Update fixed vars
    for line_item in program_item.iter_mut() {
        match line_item {
            LineItemMut::Branch(_) => {
                // unimplemented!();
            },
            LineItemMut::Stmt(stmt) => {
                match stmt {
                    Stmt::Fix(names) => {
                        for name in names.iter() {
                            fixed_map.insert(name.clone(), true);
                        }
                    },
                    Stmt::Asn(Lv::Var(Var { key, fixed }), _) => {
                        *fixed = *fixed || *fixed_map.get(key).unwrap_or(&false);
                    },
                    _ => {},
                }
                // unimplemented!();
            },
        }
    }

    Ok(program_item)
}

#[derive(Copy, Clone, Debug)]
pub enum Alias {
    Num(f64),
    Var(bool),
}

#[derive(Clone, Debug)]
pub struct MypsLexer {
    pub var_flags: BTreeMap<String, (usize, u8)>,
}

const VAR_ASSIGNED: u8 = 1;
const VAR_READ: u8 = 2;

impl MypsLexer {
    pub fn new() -> Self {
        let var_flags = BTreeMap::new();
        Self { var_flags }
    }

    pub fn parse_and_lex_program_str(
        &mut self,
        source: &str,
        indent_size: usize,
    ) -> MypsResult<Item> {
        let mut program_item = self.parse_program_str(source, indent_size).unwrap();
        let mut aliases = BTreeMap::new();
        self.lex_item(&mut program_item, &mut aliases).unwrap();
        Ok(program_item)
    }

    pub fn parse_program_str(&mut self, source: &str, indent_size: usize) -> MypsResult<Item> {
        let mut block_stack: Vec<(Block, Option<String>)> =
            vec![(Block::new(Branch::Program), None)];
        let mut indent_stack = vec![0_usize];
        let mut curr_indent = 0_usize;
        let mut expect_indent = false;
        // Parse lines and lex scopes and indentations
        for (i, line_src) in source.trim_end().split("\n").enumerate() {
            // Parse line string into a pair
            let pair = MypsParser::parse(Rule::single_line, line_src)
                .unwrap()
                .only_rule(Rule::single_line, "a line")
                .unwrap();
            // Parse line pair into number of left-spaces and an Item
            let (spaces, item) = Item::try_from_pair(pair).unwrap();
            // Lex scopes and indentation

            if !matches!(item, Item::Stmt(Stmt::Empty, _)) {
                let indent = if spaces % indent_size != 0 {
                    panic!("Invalid indent size {} (line {})", spaces, i);
                } else {
                    spaces / indent_size
                };
                if expect_indent {
                    if indent <= curr_indent {
                        panic!(
                            "Expected indent (line {}, {}, {}, {:?})",
                            i, indent, curr_indent, indent_stack
                        );
                    } else {
                        indent_stack.push(indent);
                        curr_indent = indent;
                        expect_indent = false;
                    }
                } else {
                    if indent < *indent_stack.last().unwrap() {
                        // Remove empties from previous head
                        let empties = {
                            let (head, _) = block_stack.last_mut().unwrap();
                            let i = 1 + head
                                .items
                                .iter()
                                .rposition(|item| !matches!(item, Item::Stmt(Stmt::Empty, _)))
                                .unwrap_or(head.items.len());
                            head.items.split_off(i)
                        };
                        // Pop indents and blocks until at new head
                        while indent < *indent_stack.last().unwrap() {
                            indent_stack.pop();
                            let (block, comment_opt) = block_stack.pop().unwrap();
                            let (head, _) = block_stack.last_mut().unwrap();
                            head.items.push(Item::Block(block, comment_opt));
                            curr_indent = *indent_stack.last().unwrap();
                        }
                        // Added removed empties to new head
                        let (head, _) = block_stack.last_mut().unwrap();
                        head.items.extend(empties);
                    }
                }
            }
            // Place item
            match item {
                Item::Block(block, comment_opt) => {
                    block_stack.push((block, comment_opt));
                    expect_indent = true;
                }
                Item::Stmt(..) => {
                    if !matches!(item, Item::Stmt(Stmt::Empty, _)) {
                        expect_indent = false;
                    }
                    match item {
                        _ => {}
                    }
                    let (head, _) = block_stack.last_mut().unwrap();
                    head.items.push(item);
                }
            }
        }
        if expect_indent {
            panic!("Expected indent");
        }
        while block_stack.len() > 1 {
            // indent_stack.pop();
            let (block, comment_opt) = block_stack.pop().unwrap();
            let (head, _) = block_stack.last_mut().unwrap();
            head.items.push(Item::Block(block, comment_opt));
        }
        let (program_block, _) = block_stack.pop().unwrap();
        let program_item = Item::Block(program_block, None);
        Ok(program_item)
    }

    pub fn lex_item(
        &mut self,
        item: &mut Item,
        aliases: &mut BTreeMap<String, Alias>,
    ) -> Result<(), String> {
        match item {
            Item::Block(Block { branch, items }, _) => {
                let mut aliases = aliases.clone();
                match branch {
                    Branch::While { cond } => {
                        unimplemented!();
                    }
                    Branch::If { cond, .. } => {
                        unimplemented!();
                    }
                    Branch::Elif { cond, .. } => {
                        unimplemented!();
                    }
                    Branch::For(var, s, e, step) => {
                        unimplemented!();
                    }
                    _ => {}
                }
                for (j, item) in items.iter_mut().enumerate() {
                    self.lex_item(item, &mut aliases).unwrap();
                }
            }
            Item::Stmt(stmt, _) => match stmt {
                Stmt::Fix(names) => {
                    // for name in names.iter() {
                    //     self.var_flags.try_insert(name.clone(), (i, 0)).ok();
                    // }
                }
                Stmt::Asn(lv, rv) => {
                    if let Lv::Var(Var { key, fixed }) = lv {
                        let fixed = *fixed
                            || aliases
                                .get(key)
                                .map(|alias| {
                                    if let Alias::Var(fixed) = alias {
                                        *fixed
                                    } else {
                                        false
                                    }
                                })
                                .unwrap_or(false);

                        // if self.var_flags.contains_key(key) {
                        //     let (_, flag) = self.var_flags.get_mut(key).unwrap();
                        //     *flag |= VAR_ASSIGNED;
                        // } else {
                        //     self.var_flags.insert(key.clone(), (i, VAR_ASSIGNED));
                        // }
                        aliases.insert(key.clone(), Alias::Var(fixed));
                    }
                }
                Stmt::SelfAsn { lhs, rhs, .. } => {
                    unimplemented!();
                }
                _ => {}
            },
        }
        Ok(())
    }

    fn lex_rv(&mut self, rv: &Rv) {
        match rv {
            Rv::Expr(expr) => {
                unimplemented!();
            }
            Rv::Dev(dev) => {
                unimplemented!();
            }
            Rv::Var(var) => {
                unimplemented!();
            }
        }
    }

    fn lex_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Num(num) => {
                unimplemented!();
            }
            Expr::Unary { op, rhs } => {
                unimplemented!();
            }
            Expr::Binary { op, lhs, rhs } => {
                unimplemented!();
            }
            Expr::Ternary { cond, if_t, if_f } => {
                unimplemented!();
            }
        }
    }

    fn lex_num(&mut self, num: &Num) {
        match num {
            Num::Lit(n) => {
                unimplemented!();
            }
            Num::Var(var) => {
                unimplemented!();
            }
            Num::Expr(box expr) => {
                unimplemented!();
            }
            Num::Func(box func) => {
                unimplemented!();
            }
            Num::DevParam { dev, param } => {
                unimplemented!();
            }
            Num::DevSlot {
                dev, slot: box num, ..
            } => {
                unimplemented!();
            }
            Num::DevReagent {
                dev,
                mode: box mode,
                ..
            } => {
                unimplemented!();
            }
            Num::NetParam {
                hash: box hash,
                mode: box mode,
                ..
            } => {
                unimplemented!();
            }
        }
    }
}
