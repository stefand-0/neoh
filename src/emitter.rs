use crate::ast::*;

pub struct Emitter { pub output: String }

impl Emitter {
    pub fn new() -> Self { Self { output: String::new() } }
    pub fn emit_file(&mut self, file: &File) {
        self.output.push_str("`timescale 1ns / 1ps\n\n");
        for stmt in &file.statements {
            match stmt {
                Statement::Block(b) => self.emit_block(b),
                Statement::Testbench(t) => self.emit_testbench(t),
                Statement::Testgroup(g) => self.emit_testgroup(g),
                Statement::Piece(p) => self.emit_piece(p),
                Statement::Known(_, _) => {},
            }
        }
    }

    fn emit_block(&mut self, b: &BlockDef) {
        self.output.push_str(&format!("module {}();\n", b.name));
        for stmt in &b.body {
            match stmt {
                BlockStmt::RetAssign(r) => {
                    let w = r.width.as_ref().map(|x| format!("[{}:{}]", x.msb, x.lsb)).unwrap_or_default();
                    self.output.push_str(&format!("  assign {} {} = {};\n", w, r.target, r.expr));
                }
                BlockStmt::PassParams(p) => {
                    self.output.push_str(&format!("  {} {}({});\n", p.block_type, p.inst_name, p.params.join(", ")));
                }
                BlockStmt::RetVar(v) => { self.output.push_str(&format!("  reg {};\n", v)); }
                BlockStmt::NestedBlock(nb) => self.emit_block(nb),
            }
        }
        self.output.push_str("endmodule\n");
    }

    fn emit_piece(&mut self, p: &PieceDef) {
        self.output.push_str(&format!("interface {};\n", p.name));
        for (dir, name) in &p.members { self.output.push_str(&format!("  {} logic {};\n", dir, name)); }
        self.output.push_str("endinterface\n");
    }

    fn emit_testbench(&mut self, t: &TestbenchDef) {
        self.output.push_str(&format!("module tb_{}();\n", t.name));
        self.output.push_str(&format!("  {} dut();\n", t.target));
        self.output.push_str("  initial begin\n");
        for cmd in &t.body { self.emit_verif_cmd(cmd); }
        self.output.push_str("  end\nendmodule\n");
    }

    fn emit_verif_cmd(&mut self, cmd: &VerifCmd) {
        match cmd {
            VerifCmd::Expect { time, lhs, rhs } => self.output.push_str(&format!("    #{} assert({} == {});\n", time, lhs, rhs)),
            VerifCmd::Pulse { len, gap } => self.output.push_str(&format!("    {} = 1; #{} {} = 0; #{} {} = 0;\n", len, len, len, gap, gap)),
            VerifCmd::Watchfor { lhs, rhs, time_b, out, .. } => {
                let out_sv = match out { OutTarget::Variable(v) => v.clone(), OutTarget::Literal(s) => s.clone() };
                self.output.push_str(&format!("    wait({} == {}); #{} ({});\n", lhs, rhs, time_b, out_sv));
            }
            VerifCmd::Write(w) => self.output.push_str(&format!("    {} = {};\n", w.target, w.val)),
            VerifCmd::Put(p) => self.emit_put(p),
        }
    }

    fn emit_put(&mut self, p: &PutStmt) {
        self.output.push_str(&format!("    {} {} {};\n", p.target, p.op, p.expr));
    }

    fn emit_testgroup(&mut self, g: &TestGroupDef) {
        self.output.push_str(&format!("module tg_{}();\n", g.name));
        for item in &g.items {
            match item {
                GroupItem::Do(name) => self.output.push_str(&format!("  initial begin\n    #0;\n    tb_{} inst();\n  end\n", name)),
                GroupItem::Same(names) => {
                    self.output.push_str("  fork\n");
                    for name in names { self.output.push_str(&format!("    begin tb_{} inst(); end\n", name)); }
                    self.output.push_str("  join\n");
                }
            }
        }
        self.output.push_str("endmodule\n");
    }
}
