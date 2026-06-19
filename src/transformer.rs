/*
 * Copyright 2026 stefand-0
 * Licensed under the Apache License, Version 2.0 (the "License");
 */

use crate::ast::*;
use crate::parser::Rule;
use pest::iterators::{Pair, Pairs};

pub fn build_ast(pairs: Pairs<Rule>) -> File {
    let mut statements = Vec::new();
    for pair in pairs {
        if pair.as_rule() == Rule::file {
            for p in pair.into_inner() {
                match p.as_rule() {
                    Rule::block_def => statements.push(Statement::Block(build_block(p))),
                    Rule::testbench => statements.push(Statement::Testbench(build_testbench(p))),
                    Rule::testgroup => statements.push(Statement::Testgroup(build_testgroup(p))),
                    Rule::piece_def => statements.push(Statement::Piece(build_piece(p))),
                    _ => (),
                }
            }
        }
    }
    File { statements }
}

fn build_block(pair: Pair<Rule>) -> BlockDef {
    let mut inner = pair.into_inner();
    let name = inner.next().expect("Expected block name").as_str().to_string();
    let params = Vec::new(); // Simplified for brevity; implement param parsing here
    let mut body = Vec::new();
    for p in inner {
        match p.as_rule() {
            Rule::ret_stmt => body.push(BlockStmt::RetAssign(build_ret_assign(p))),
            Rule::ret_var_stmt => body.push(BlockStmt::RetVar(p.into_inner().next().unwrap().as_str().to_string())),
            _ => (),
        }
    }
    BlockDef { name, params, body }
}

fn build_ret_assign(pair: Pair<Rule>) -> RetAssign {
    let mut i = pair.into_inner();
    let target = i.next().unwrap().as_str().to_string();
    let expr = i.last().unwrap().as_str().to_string();
    RetAssign { target, expr, width: None }
}

fn build_testbench(pair: Pair<Rule>) -> TestbenchDef {
    let mut i = pair.into_inner();
    let name = i.next().unwrap().as_str().to_string();
    let target = i.next().unwrap().as_str().to_string();
    i.next(); // Skip getvars
    let body_nodes = i.nth(1).unwrap().into_inner();
    let body = body_nodes.map(build_verif_cmd).collect();
    TestbenchDef { name, target, body }
}

fn build_verif_cmd(pair: Pair<Rule>) -> VerifCmd {
    match pair.as_rule() {
        Rule::expect_cmd => {
            let mut i = pair.into_inner();
            VerifCmd::Expect {
                time: i.next().unwrap().as_str().parse().unwrap(),
                lhs: i.next().unwrap().as_str().to_string(),
                rhs: i.nth(1).unwrap().as_str().to_string(),
            }
        },
        Rule::pulse_cmd => {
            let mut i = pair.into_inner();
            VerifCmd::Pulse {
                len: i.next().unwrap().as_str().to_string(),
                gap: i.nth(1).unwrap().as_str().to_string(),
            }
        },
        Rule::out_cmd => {
            let mut i = pair.into_inner();
            let time = i.next().unwrap().as_str().parse().unwrap();
            let arg = i.next().unwrap();
            let target = if arg.as_rule() == Rule::string_literal {
                OutTarget::Literal(arg.as_str().to_string())
            } else {
                OutTarget::Variable(arg.as_str().to_string())
            };
            VerifCmd::Out { time, target }
        },
        Rule::writefile_cmd => {
            let mut i = pair.into_inner();
            VerifCmd::WriteFile {
                mode: i.next().unwrap().as_str().to_string(),
                file: i.nth(1).unwrap().as_str().to_string(),
            }
        },
        Rule::put_stmt => {
            let mut i = pair.into_inner();
            VerifCmd::Put(PutStmt {
                target: i.next().unwrap().as_str().to_string(),
                op: i.next().unwrap().as_str().to_string(),
                expr: i.next().unwrap().as_str().to_string(),
                width: None,
            })
        },
        _ => unreachable!(),
    }
}

fn build_testgroup(pair: Pair<Rule>) -> TestGroupDef {
    let mut i = pair.into_inner();
    TestGroupDef {
        name: i.next().unwrap().as_str().to_string(),
        items: Vec::new(),
    }
}

fn build_piece(pair: Pair<Rule>) -> PieceDef {
    let mut i = pair.into_inner();
    PieceDef {
        name: i.next().unwrap().as_str().to_string(),
        members: Vec::new(),
    }
}

