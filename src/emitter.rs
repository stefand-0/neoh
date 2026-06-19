/*
 * Copyright 2026 stefand-0
 * Licensed under the Apache License, Version 2.0 (the "License");
 */

use crate::ast::{
    BlockDef, BlockStmt, File, GroupItem, OutTarget, PieceDef, PutStmt, Statement,
    TestGroupDef, TestbenchDef, VerifCmd
};

pub fn emit(file: &File) -> String {
    let mut emitter = Emitter::new();
    emitter.emit_file(file);
    emitter.output
}

pub struct Emitter {
    pub output: String,
}

impl Emitter {
    pub fn new() -> Self {
        Emitter { output: String::new() }
    }

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
            self.output.push_str("\n");
        }
    }

    fn emit_block(&mut self, b: &BlockDef) {
        self.output.push_str(&format!("module {}();\n", b.name));
        for stmt in &b.body {
            match stmt {
                BlockStmt::RetAssign(r) => {
                    self.output.push_str(&format!("  assign {} = {};\n", r.target, r.expr));
                }
                BlockStmt::RetVar(v) => {
                    self.output.push_str(&format!("  // Return variable tracking: {}\n", v));
                }
                BlockStmt::PassParams(p) => {
                    self.output.push_str(&format!("  {} {} (\n", p.block_type, p.inst_name));
                    let formatted: Vec<String> = p.params.iter().map(|param| format!("    .{}()", param)).collect();
                    self.output.push_str(&formatted.join(",\n"));
                    self.output.push_str("\n  );\n");
                }
            }
        }
        self.output.push_str("endmodule\n");
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
                self.output.push_str(&format!("    #{} if ({} !== {}) $display(\"Assertion failed for {}\");\n", time, lhs, rhs, lhs));
            }
            VerifCmd::Pulse { len, gap } => {
                self.output.push_str(&format!("    // Pulse generated with length {} and gap {}\n", len, gap));
            }
            VerifCmd::Watchfor { time_a, lhs, rhs, time_b, out } => {
                let target_str = match out {
                    OutTarget::Variable(v) => v.clone(),
                    OutTarget::Literal(s) => format!("\"{}\"", s),
                };
                self.output.push_str(&format!("    // Watch for {} == {} between {} and {}, output to {}\n", lhs, rhs, time_a, time_b, target_str));
            }
            VerifCmd::Write(w) => {
                self.output.push_str(&format!("    {} = {};\n", w.target, w.val));
            }
            VerifCmd::Put(p) => self.emit_put(p),
            VerifCmd::Out { time, target } => {
                let target_str = match target {
                    OutTarget::Variable(v) => v.clone(),
                    OutTarget::Literal(s) => format!("\"{}\"", s),
                };
                self.output.push_str(&format!("    #{} $display({});\n", time, target_str));
            }
            VerifCmd::WriteFile { mode, file } => {
                self.output.push_str(&format!("    // File operation - Mode: {}, Target File: {}\n", mode, file));
            }
        }
    }

    fn emit_put(&mut self, p: &PutStmt) {
        self.output.push_str(&format!("    {} {} {};\n", p.target, p.op, p.expr));
    }

    fn emit_testgroup(&mut self, g: &TestGroupDef) {
        self.output.push_str(&format!("module tg_{}();\n", g.name));
        for item in &g.items {
            match item {
                GroupItem::Do(name) => {
                    self.output.push_str(&format!("  tb_{} instance_{}();\n", name, name));
                }
                GroupItem::Same(members) => {
                    self.output.push_str("  // Concurrent execution block\n");
                    for m in members {
                        self.output.push_str(&format!("  tb_{} instance_{}();\n", m, m));
                    }
                }
            }
        }
        self.output.push_str("endmodule\n");
    }

    fn emit_piece(&mut self, _p: &PieceDef) {
        self.output.push_str("// Piece structure definition generated in AST\n");
    }
}

