use crate::ast::*;

pub struct Emitter {
    pub output: String,
    in_aff: bool,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            in_aff: false,
        }
    }

    pub fn emit_file(&mut self, file: &File) {
        for stmt in &file.statements {
            self.emit_statement(stmt);
        }
    }

    fn emit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Block(b) => self.emit_block(b),
            Statement::Known(_) => self.output.push_str("// Known macro generated globally\n"),
            _ => unimplemented!("Emitter logic for this statement type not yet implemented"),
        }
    }

    fn emit_block(&mut self, block: &BlockDef) {
        self.output.push_str(&format!("module {}();\n", block.name));
        
        for stmt in &block.body {
            match stmt {
                BlockStmt::Put(p) => self.emit_put(p),
                BlockStmt::Aff(a) => self.emit_aff(a),
                _ => {}
            }
        }
        
        self.output.push_str("endmodule\n");
    }

    fn emit_aff(&mut self, aff: &AffBlock) {
        self.in_aff = true;
        let sens = aff.sensitivity.join(" or ");
        self.output.push_str(&format!("always_ff @({}) begin\n", sens));
        
        for stmt in &aff.body {
            self.emit_put(stmt);
        }
        
        self.output.push_str("end\n");
        self.in_aff = false;
    }

    fn emit_put(&mut self, stmt: &PutStmt) {
        let width = match &stmt.width {
            Some(w) => format!("{}:{}", w.msb, w.lsb),
            None => "".to_string(),
        };

        if self.in_aff {
            // Non-blocking assignment
            self.output.push_str(&format!("  {} <= {}'({});\n", stmt.target, width, stmt.expr));
        } else {
            // Continuous assignment
            self.output.push_str(&format!("assign {} = {}'({});\n", stmt.target, width, stmt.expr));
        }
    }
}
