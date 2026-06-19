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


#[derive(Debug, Clone, PartialEq)]
pub struct File { pub statements: Vec<Statement> }

#[derive(Debug, Clone, PartialEq)]
pub enum OutTarget {
    Variable(String),
    Literal(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Known(String, Vec<String>),
    Block(BlockDef),
    Piece(PieceDef),
    Testbench(TestbenchDef),
    Testgroup(TestGroupDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<BlockStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockStmt {
    RetAssign(RetAssign),
    PassParams(PassParams),
    RetVar(String),
    NestedBlock(BlockDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetAssign {
    pub target: String,
    pub expr: String,
    pub width: Option<WidthConstraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WidthConstraint { pub msb: u32, pub lsb: u32 }

#[derive(Debug, Clone, PartialEq)]
pub struct PassParams {
    pub inst_name: String,
    pub block_type: String,
    pub params: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PieceDef { pub name: String, pub members: Vec<(String, String)> }

#[derive(Debug, Clone, PartialEq)]
pub struct TestbenchDef {
    pub name: String,
    pub target: String,
    pub body: Vec<VerifCmd>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerifCmd {
    Expect { time: u32, lhs: String, rhs: String },
    Pulse { len: String, gap: String },
    Watchfor { time_a: u32, lhs: String, rhs: String, time_b: u32, out: String },
    WriteFile { mode: String, file: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestGroupDef { pub name: String, pub items: Vec<GroupItem> }

#[derive(Debug, Clone, PartialEq)]
pub enum GroupItem { Do(String), Same(Vec<String>) }
