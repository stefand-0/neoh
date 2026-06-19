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

pub struct Emitter {
    pub output: String,
}

impl Emitter {
    pub fn new() -> Self {
        Self { output: String::new() }
    }

    pub fn emit_file(&mut self, file: &File) {
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
                BlockStmt::RetVar(v) => {
                    self.output.push_str(&format!("  reg {};\n", v));
                }
                BlockStmt::NestedBlock(nb) => self.emit_block(nb),
            }
        }
        self.output.push_str("endmodule\n");
    }

    fn emit_piece(&mut self, p: &PieceDef) {
        self.output.push_str(&format!("interface {};\n", p.name));
        for (dir, name) in &p.members {
            self.output.push_str(&format!("  {} logic {};\n", dir, name));
        }
        self.output.push_str("endinterface\n");
    }

    fn emit_testbench(&mut self, t: &TestbenchDef) {
        self.output.push_str(&format!("module tb_{}();\n", t.name));
        self.output.push_str(&format!("  {} dut();\n", t.target));
        self.output.push_str("  initial begin\n");
        for cmd in &t.body {
            self.emit_verif_cmd(cmd);
        }
        self.output.push_str("  end\n");
        self.output.push_str("endmodule\n");
    }

    fn emit_verif_cmd(&mut self, cmd: &VerifCmd) {
        match cmd {
            VerifCmd::Expect { time, lhs, rhs } => {
                self.output.push_str(&format!("    #{} assert({} == {});\n", time, lhs, rhs));
            }
            VerifCmd::Pulse { len, gap } => {
                self.output.push_str(&format!("    {} = 1; #{} {} = 0; #{} {} = 1;\n", len, len, len, gap, gap));
            }
            VerifCmd::Watchfor { time_a, lhs, rhs, time_b, out } => {
                let out_sv = match out {
                    OutTarget::Variable(v) => v.clone(),
                    OutTarget::Literal(s) => s.clone(),
                };
                self.output.push_str(&format!("    wait({} == {}); #{} $display({});\n", lhs, rhs, time_b, out_sv));
            }
            VerifCmd::WriteFile { mode, file } => {
                self.output.push_str(&format!("    $dumpfile(\"{}\"); $dumpvars(0, {});\n", file, mode));
            }
            VerifCmd::Put(p) => self.emit_put(p),
        }
    }

    fn emit_put(&mut self, p: &PutStmt) {
        // Assuming non-blocking assignment for testbench driving
        self.output.push_str(&format!("    {} {} {};\n", p.target, p.op, p.expr));
    }

    fn emit_testgroup(&mut self, g: &TestGroupDef) {
        self.output.push_str(&format!("module tg_{}();\n", g.name));
        for item in &g.items {
            match item {
                GroupItem::Do(name) => {
                    self.output.push_str(&format!("  initial begin\n    #0;\n    tb_{} inst();\n  end\n", name));
                }
                GroupItem::Same(names) => {
                    self.output.push_str("  fork\n");
                    for name in names {
                        self.output.push_str(&format!("    begin tb_{} inst(); end\n", name));
                    }
                    self.output.push_str("  join\n");
                }
            }
        }
        self.output.push_str("endmodule\n");
    }
}
