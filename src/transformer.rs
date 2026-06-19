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
                    Rule::known_stmt => (), // Handle if needed
                    _ => (),
                }
            }
        }
    }
    File { statements }
}

fn build_block(pair: Pair<Rule>) -> BlockDef {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    
    let mut params = Vec::new();
    let mut body = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::block_params => {
                params = p.into_inner().map(|x| x.as_str().to_string()).collect();
            }
            Rule::ret_stmt => body.push(BlockStmt::RetAssign(build_ret_assign(p))),
            Rule::ret_var_stmt => {
                if let Some(n) = p.into_inner().next() {
                    body.push(BlockStmt::RetVar(n.as_str().to_string()));
                }
            }
            Rule::pass_stmt => {
                 // Extend this if you use pass_stmt
            }
            _ => (),
        }
    }
    BlockDef { name, params, body }
}

fn build_testbench(pair: Pair<Rule>) -> TestbenchDef {
    let mut i = pair.into_inner();
    let name = i.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let target = i.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    
    let mut body = Vec::new();
    for p in i {
        if p.as_rule() == Rule::verif_cmd {
            body.push(build_verif_cmd(p));
        } else if let Some(inner) = p.into_inner().find(|x| x.as_rule() == Rule::verif_cmd) {
            body.push(build_verif_cmd(inner));
        }
    }

    TestbenchDef { name, target, body }
}

fn build_verif_cmd(pair: Pair<Rule>) -> VerifCmd {
    let inner = pair.into_inner().next().expect("verif_cmd missing inner rule");
    match inner.as_rule() {
        Rule::expect_cmd => {
            let mut i = inner.into_inner();
            VerifCmd::Expect {
                time: i.next().map(|x| x.as_str().parse().unwrap_or(0)).unwrap_or(0),
                lhs: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
                rhs: i.nth(1).map(|x| x.as_str().to_string()).unwrap_or_default(),
            }
        },
        Rule::pulse_cmd => {
            let mut i = inner.into_inner();
            VerifCmd::Pulse {
                len: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
                gap: i.nth(1).map(|x| x.as_str().to_string()).unwrap_or_default(),
            }
        },
        Rule::watch_cmd => {
            let mut i = inner.into_inner();
            VerifCmd::Watchfor {
                time_a: i.next().map(|x| x.as_str().parse().unwrap_or(0)).unwrap_or(0),
                lhs: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
                rhs: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
                time_b: i.next().map(|x| x.as_str().parse().unwrap_or(0)).unwrap_or(0),
                out: OutTarget::Variable(i.next().map(|x| x.as_str().to_string()).unwrap_or_default()),
            }
        },
        Rule::out_cmd => {
            let mut i = inner.into_inner();
            let time = i.next().map(|x| x.as_str().parse().unwrap_or(0)).unwrap_or(0);
            let arg = i.next().expect("Missing arg in out_cmd");
            let target = if arg.as_rule() == Rule::string_literal {
                OutTarget::Literal(arg.as_str().trim_matches('"').to_string())
            } else {
                OutTarget::Variable(arg.as_str().to_string())
            };
            VerifCmd::Out { time, target }
        },
        Rule::writefile_cmd => {
            let mut i = inner.into_inner();
            VerifCmd::WriteFile {
                mode: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
                file: i.nth(1).map(|x| x.as_str().to_string()).unwrap_or_default(),
            }
        },
        Rule::write_cmd => {
            let mut i = inner.into_inner();
            VerifCmd::Write(WriteStmt {
                target: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
                val: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
            })
        },
        Rule::put_stmt => build_put(inner),
        _ => unreachable!("Unhandled verif_cmd rule: {:?}", inner.as_rule()),
    }
}

fn build_put(pair: Pair<Rule>) -> VerifCmd {
    let mut i = pair.into_inner();
    VerifCmd::Put(PutStmt {
        target: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
        op: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
        expr: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
        width: None,
    })
}

fn build_ret_assign(pair: Pair<Rule>) -> RetAssign {
    let mut i = pair.into_inner();
    RetAssign {
        target: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
        expr: i.last().map(|x| x.as_str().to_string()).unwrap_or_default(),
        width: None,
    }
}

fn build_testgroup(pair: Pair<Rule>) -> TestGroupDef {
    let mut i = pair.into_inner();
    let name = i.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    
    let mut items = Vec::new();
    for p in i {
        match p.as_rule() {
            Rule::do_cmd => {
                let name = p.into_inner().next().map(|n| n.as_str().to_string()).unwrap_or_default();
                items.push(TestGroupItem::Do(name));
            }
            Rule::same_block => {
                // Collect all identifiers inside the same {} block
                let members = p.into_inner().map(|x| x.as_str().to_string()).collect();
                items.push(TestGroupItem::Same(members));
            }
            _ => (),
        }
    }
    TestGroupDef { name, items }
}


fn build_piece(pair: Pair<Rule>) -> PieceDef {
    let mut i = pair.into_inner();
    PieceDef {
        name: i.next().map(|x| x.as_str().to_string()).unwrap_or_default(),
        members: Vec::new(),
    }
}

