#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Known(KnownDef),
    Block(BlockDef),
    Piece(PieceDef),
    Testbench(TestbenchDef),
    Testgroup(TestGroupDef),
}

// --- Blocks & Definitions ---

#[derive(Debug, Clone, PartialEq)]
pub struct BlockDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<BlockStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockStmt {
    Put(PutStmt),
    Aff(AffBlock),
    PieceInst(PieceInst),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AffBlock {
    pub sensitivity: Vec<String>,
    pub body: Vec<PutStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PutStmt {
    pub target: String,
    pub op: String, // "=" or "<="
    pub expr: String,
    pub width: Option<WidthConstraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WidthConstraint {
    pub msb: u32,
    pub lsb: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PieceDef {
    pub name: String,
    pub members: Vec<PieceMember>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PieceMember {
    pub direction: String, // "in" or out"
    pub msb: u32,
    pub lsb: u32,
    pub name: String,
}

// --- Verification  ---

#[derive(Debug, Clone, PartialEq)]
pub struct TestbenchDef {
    pub name: String,
    pub target: String,
    pub getvars: Vec<String>,
    pub body: Vec<VerifCmd>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerifCmd {
    Expect { time: u32, lhs: String, rhs: String },
    Pulse { signal: String, length: u32 },
    WriteFile { mode: String, file: String },
    Watchfor { time: u32, lhs: String, rhs: String, action: Option<WatchAction> },
    Out(String),
    Put(PutStmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WatchAction {
    pub delay: u32,
    pub signal: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestGroupDef {
    pub name: String,
    pub items: Vec<GroupItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupItem {
    Do(String),
    Same(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnownDef {
    pub signals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PieceInst {
    pub type_name: String,
    pub instance_name: String,
}
