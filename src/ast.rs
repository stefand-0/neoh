// ast.rs

#[derive(Debug, Clone, PartialEq)]
pub struct File { pub statements: Vec<Statement> }

#[derive(Debug, Clone, PartialEq)]
pub enum OutTarget { Variable(String), Literal(String) }

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Known(String, Vec<String>),
    Block(BlockDef),
    Piece(PieceDef),
    Testbench(TestbenchDef),
    Testgroup(TestGroupDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockDef { pub name: String, pub params: Vec<String>, pub body: Vec<BlockStmt> }

#[derive(Debug, Clone, PartialEq)]
pub enum BlockStmt {
    RetAssign(RetAssign),
    PassParams(PassParams),
    RetVar(String),
    NestedBlock(BlockDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetAssign { pub target: String, pub expr: String, pub width: Option<WidthConstraint> }

#[derive(Debug, Clone, PartialEq)]
pub struct WidthConstraint { pub msb: u32, pub lsb: u32 }

#[derive(Debug, Clone, PartialEq)]
pub struct PassParams { pub inst_name: String, pub block_type: String, pub params: Vec<String> }

#[derive(Debug, Clone, PartialEq)]
pub struct PieceDef { pub name: String, pub members: Vec<(String, String)> }

#[derive(Debug, Clone, PartialEq)]
pub struct TestbenchDef { pub name: String, pub target: String, pub body: Vec<VerifCmd> }

#[derive(Debug, Clone, PartialEq)]
pub enum VerifCmd {
    Expect { time: u32, lhs: String, rhs: String },
    Pulse { len: String, gap: String },
    Watchfor { time_a: u32, lhs: String, rhs: String, time_b: u32, out: OutTarget },
    Write(WriteStmt),
    Put(PutStmt),
    Out { time: u32, target: OutTarget },          // Added
    WriteFile { mode: String, file: String },      // Added
}

#[derive(Debug, Clone, PartialEq)]
pub struct WriteStmt { pub target: String, pub val: String }

#[derive(Debug, Clone, PartialEq)]
pub struct PutStmt { pub target: String, pub op: String, pub expr: String, pub width: Option<WidthConstraint> }

#[derive(Debug, Clone, PartialEq)]
pub struct TestGroupDef { pub name: String, pub items: Vec<GroupItem> }

#[derive(Debug, Clone, PartialEq)]
pub enum GroupItem { Do(String), Same(Vec<String>) }

#[derive(Debug, Clone, PartialEq)]
pub enum TestGroupItem {
    Do(String),
    Same(Vec<String>),
}
