use std::env;
use std::fs;
use std::process;
use pest::Parser;

mod ast;
mod emitter;
mod parser; // parser struct 

use crate::parser::{NeoParser, Rule};
use crate::emitter::Emitter;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: neo_compiler <input_file.neoh>");
        process::exit(1);
    }

    let input_path = &args[1];

    // Read the source code
    let source_code = fs::read_to_string(input_path).expect("Could not read file");

    // 3Parse the source code
    let pairs = NeoParser::parse(Rule::file, &source_code).unwrap_or_else(|e| {
        eprintln!("Parsing Error: {}", e);
        process::exit(1);
    });

    // Build AST `src/transformer.rs`)
    // This is the bridge between the raw pest 'Pairs' and your 'ast.rs' structs
    let ast_data = transformer::build_ast(pairs);

    // Emit SystemVerilog
    let mut emitter = Emitter::new();
    emitter.emit_file(&ast_data);

    // Write output
    let output_path = input_path.replace(".neoh", ".sv");
    fs::write(&output_path, emitter.output).expect("Could not write SV file");

    println!("Successfully compiled {} -> {}", input_path, output_path);
}
