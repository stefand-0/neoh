/*
 * Copyright 2026 stefand-0
 * Licensed under the Apache License, Version 2.0 (the "License");
 */

#[derive(Debug, Clone)]
pub struct File {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Block(BlockDef),
    Testbench(TestbenchDef),
    Testgroup(TestGroupDef),
    Piece(PieceDef),
    Known(String, String),
}

#[derive(Debug, Clone)]
pub struct Width {
    pub msb: i32,
    pub lsb: i32,
}

#[derive(Debug, Clone)]
pub struct BlockDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<BlockStmt>,
}

#[derive(Debug, Clone)]
pub enum BlockStmt {
    RetAssign(RetAssign),
    RetVar(String),
    PassParams(PassParams),
}

#[derive(Debug, Clone)]
pub struct RetAssign {
    pub target: String,
    pub expr: String,
    pub width: Option<Width>,
}

#[derive(Debug, Clone)]
pub struct PassParams {
    pub block_type: String,
    pub inst_name: String,
    pub params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TestbenchDef {
    pub name: String,
    pub target: String,
    pub body: Vec<VerifCmd>,
}

#[derive(Debug, Clone)]
pub enum OutTarget {
    Variable(String),
    Literal(String),
}

#[derive(Debug, Clone)]
pub struct WriteStmt {
    pub target: String,
    pub val: String,
}

#[derive(Debug, Clone)]
pub struct PutStmt {
    pub target: String,
    pub op: String,
    pub expr: String,
    pub width: Option<Width>,
}

#[derive(Debug, Clone)]
pub enum VerifCmd {
    Expect {
        time: u32,
        lhs: String,
        rhs: String,
    },
    Pulse {
        len: String,
        gap: String,
    },
    Watchfor {
        time_a: u32,
        lhs: String,
        rhs: String,
        time_b: u32,
        out: OutTarget,
    },
    Write(WriteStmt),
    Put(PutStmt),
    Out {
        time: u32,
        target: OutTarget,
    },
    WriteFile {
        mode: String,
        file: String,
    },
}

#[derive(Debug, Clone)]
pub enum GroupItem {
    Do(String),
    Same(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct TestGroupDef {
    pub name: String,
    pub items: Vec<GroupItem>,
}

#[derive(Debug, Clone)]
pub struct PieceDef {
    pub name: String,
    pub members: Vec<String>,
}

