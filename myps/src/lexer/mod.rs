use std::collections::BTreeMap;

use pest::Parser;

use ast_traits::{AstError, AstNode, AstPair, AstPairs, IntoAst};
use mips::Alias;

use crate::ast::{Block, Branch, Item, Stmt};
use crate::{MypsError, MypsParser, MypsResult, Pair, Rule};

const INDENT_SIZE: usize = 4;

// pub fn lex_str(source: &str) -> MypsResult<()> {
pub fn lex_str(source: &str) -> MypsResult<Item> {
    // let mut items = Item::block();
    let mut block_stack: Vec<(Block, Option<String>)> = vec![(Block::new(Branch::Program), None)];

    let mut indent_stack = vec![0_usize];
    let mut curr_indent = 0_usize;
    let mut expect_indent = false;

    for (i, line_src) in source.trim_end().split("\n").enumerate() {
        print!("({}) '{}'", i, line_src);
        let pair = MypsParser::parse(Rule::single_line, line_src)
            .unwrap()
            .only_rule(Rule::single_line, "a line")
            .unwrap();
        let (spaces, item) = Item::try_from_pair(pair).unwrap();

        print!(" {:?}", indent_stack);
        // Handle indent
        if !matches!(item, Item::Stmt(Stmt::Empty, _)) {
            let indent = if spaces % INDENT_SIZE != 0 {
                panic!("Invalid indent size {} (line {})", spaces, i);
            } else {
                spaces / INDENT_SIZE
            };
            if expect_indent {
                if indent <= curr_indent {
                    panic!("Expected indent (line {}, {}, {}, {:?})", i, indent, curr_indent, indent_stack);
                } else {
                    indent_stack.push(indent);
                    curr_indent = indent;
                    expect_indent = false;
                }
            } else {
                while indent < *indent_stack.last().unwrap() {
                    indent_stack.pop();
                    let (block, comment_opt) = block_stack.pop().unwrap();
                    let (head, _) = block_stack.last_mut().unwrap();
                    head.items.push(Item::Block(block, comment_opt));
                    curr_indent = *indent_stack.last().unwrap();
                }
            }
        }
        print!(" {:?}", indent_stack);

        match item {
            Item::Block(block, comment_opt) => {
                block_stack.push((block, comment_opt));
                expect_indent = true;
            }
            Item::Stmt(Stmt::Empty, ..) => {
                let (head, _) = block_stack.last_mut().unwrap();
                head.items.push(item);
            }
            Item::Stmt(..) => {
                let (head, _) = block_stack.last_mut().unwrap();
                head.items.push(item);
                expect_indent = false;
            }
        }

        println!(" {}", expect_indent);
    }
    println!("END LINES");
    if expect_indent {
        panic!("Expected indent");
    }
    // println!("expect indent {}", expect_indent);

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

#[derive(Clone, Debug)]
pub struct MypsLexer {
    pub(crate) aliases: BTreeMap<String, Alias>,
}

impl MypsLexer {
    pub fn get_alias(&self, key: &String) -> Option<&Alias> {
        self.aliases.get(key)
    }

    pub fn try_alias(&self, key: &String) -> MypsResult<&Alias> {
        self.get_alias(key).ok_or(MypsError::alias_undefined(key))
    }
}
