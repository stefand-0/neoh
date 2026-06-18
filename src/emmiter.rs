use crate::ast::*;

pub struct Emitter { pub output: String }

impl Emitter {
    pub fn new() -> Self { Self { output: String::new() } }

    pub fn emit_file(&mut self, file: &File) {
        for stmt in &file.statements {
            match stmt {
                Statement::Block(b) => self.emit_block(b),
                _ => {}
            }
        }
    }

    fn emit_block(&mut self, b: &BlockDef) {
        self.output.push_str(&format!("module {}();\n", b.name));
        for stmt in &b.body {
            match stmt {
                BlockStmt::RetAssign(r) => {
                    let w = r.width.as_ref().map(|x| format!("{}:{}", x.msb, x.lsb)).unwrap_or_default();
                    self.output.push_str(&format!("  assign {} = {}'({});\n", r.target, w, r.expr));
                },
                _ => {}
            }
        }
        self.output.push_str("endmodule\n");
    }
}
