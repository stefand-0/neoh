/*
 * Copyright 2026 stefand-0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::ast::*;
use crate::parser::Rule;
use pest::iterators::{Pair, Pairs};

pub fn build_ast(pairs: Pairs<Rule>) -> File {
    let mut statements = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::block_def => statements.push(Statement::Block(build_block(pair))),
            Rule::testbench => statements.push(Statement::Testbench(build_testbench(pair))),
            Rule::testgroup => statements.push(Statement::Testgroup(build_testgroup(pair))),
            Rule::piece_def => statements.push(Statement::Piece(build_piece(pair))),
            _ => (),
        }
    }
    File { statements }
}

fn build_block(pair: Pair<Rule>) -> BlockDef {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let mut body = Vec::new();
    
    for p in inner {
        match p.as_rule() {
            Rule::ret_stmt => body.push(BlockStmt::RetAssign(build_ret_stmt(p))),
            Rule::pass_stmt => body.push(BlockStmt::PassParams(build_pass_stmt(p))),
            Rule::ret_var_stmt => body.push(BlockStmt::RetVar(p.into_inner().next().unwrap().as_str().to_string())),
            Rule::block_def => body.push(BlockStmt::NestedBlock(build_block(p))),
            _ => (),
        }
    }
    BlockDef { name, params: vec![], body }
}

fn build_ret_stmt(pair: Pair<Rule>) -> RetAssign {
    let mut inner = pair.into_inner();
    let target = inner.next().unwrap().as_str().to_string();
    let width = inner.find_map(|p| if p.as_rule() == Rule::width_constraint { Some(parse_width(p)) } else { None });
    let expr = inner.next().unwrap().as_str().to_string();
    RetAssign { target, expr, width }
}

fn build_pass_stmt(pair: Pair<Rule>) -> PassParams {
    let mut inner = pair.into_inner();
    let inst_name = inner.next().unwrap().as_str().to_string();
    let block_type = inner.next().unwrap().as_str().to_string();
    let params = inner.map(|p| p.as_str().to_string()).collect();
    PassParams { inst_name, block_type, params }
}

fn build_piece(pair: Pair<Rule>) -> PieceDef {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let members = inner.map(|p| {
        let mut m = p.into_inner();
        (m.next().unwrap().as_str().to_string(), m.next().unwrap().as_str().to_string())
    }).collect();
    PieceDef { name, members }
}

fn build_testbench(pair: Pair<Rule>) -> TestbenchDef {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let target = inner.next().unwrap().as_str().to_string();
    
    let _getvars = inner.next().unwrap();
    let when_block = inner.next().unwrap();
    
    let mut body = Vec::new();
    for p in when_block.into_inner() {
        body.push(parse_verif_cmd(p));
    }
    
    TestbenchDef { name, target, body }
}

fn parse_verif_cmd(pair: Pair<Rule>) -> VerifCmd {
    match pair.as_rule() {
        Rule::expect_cmd => {
            let mut i = pair.into_inner();
            VerifCmd::Expect { 
                time: i.next().unwrap().as_str().parse().unwrap(), 
                lhs: i.next().unwrap().as_str().to_string(), 
                rhs: i.next().unwrap().as_str().to_string() 
            }
        },
        Rule::pulse_cmd => {
            let mut i = pair.into_inner();
            VerifCmd::Pulse { 
                len: i.next().unwrap().as_str().to_string(), 
                gap: i.next().unwrap().as_str().to_string() 
            }
        },
        Rule::watch_cmd => {
            let mut i = pair.into_inner();
            let time_a = i.next().unwrap().as_str().parse().unwrap();
            let lhs = i.next().unwrap().as_str().to_string();
            let rhs = i.next().unwrap().as_str().to_string();
            let time_b = i.next().unwrap().as_str().parse().unwrap();
            
            let out_arg = i.next().unwrap();
            let out = if out_arg.as_rule() == Rule::string_literal {
                OutTarget::Literal(out_arg.as_str().to_string())
            } else {
                OutTarget::Variable(out_arg.as_str().to_string())
            };

            VerifCmd::Watchfor { time_a, lhs, rhs, time_b, out }
        },
        Rule::write_cmd => {
            let mut i = pair.into_inner();
            VerifCmd::WriteFile { 
                mode: i.next().unwrap().as_str().to_string(), 
                file: i.next().unwrap().as_str().to_string() 
            }
        },
        Rule::put_stmt => {
            VerifCmd::Put(build_put(pair))
        },
        _ => unreachable!(),
    }
}

pub fn build_put(pair: Pair<Rule>) -> PutStmt {
    let mut inner = pair.into_inner();
    let target = inner.next().unwrap().as_str().to_string();
    let op = inner.next().unwrap().as_str().to_string();
    let expr = inner.next().unwrap().as_str().to_string();
    let width = inner.find_map(|p| if p.as_rule() == Rule::width_constraint { Some(parse_width(p)) } else { None });
    PutStmt { target, op, expr, width }
}

fn build_testgroup(pair: Pair<Rule>) -> TestGroupDef {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let mut items = Vec::new();
    for p in inner {
        match p.as_rule() {
            Rule::do_stmt => items.push(GroupItem::Do(p.into_inner().next().unwrap().as_str().to_string())),
            Rule::same_block => items.push(GroupItem::Same(p.into_inner().map(|x| x.into_inner().next().unwrap().as_str().to_string()).collect())),
            _ => (),
        }
    }
    TestGroupDef { name, items }
}

fn parse_width(pair: Pair<Rule>) -> WidthConstraint {
    let mut inner = pair.into_inner();
    WidthConstraint { msb: inner.next().unwrap().as_str().parse().unwrap(), lsb: inner.next().unwrap().as_str().parse().unwrap() }
}
