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

use std::{env, fs, process, path::Path};
use pest::Parser;

mod ast;
mod emitter;
mod parser;
mod transformer;

use crate::parser::{NeoParser, Rule};
use crate::emitter::Emitter;

fn main() {
    // 1. Collect command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: neoh <file.neoh>");
        process::exit(1);
    }

    // 2. Read source file
    let input_path = &args[1];
    let source = fs::read_to_string(input_path)
        .unwrap_or_else(|_| { 
            eprintln!("Error: Could not read file '{}'", input_path); 
            process::exit(1); 
        });

    // 3. Run Parser
    let pairs = NeoParser::parse(Rule::file, &source)
        .unwrap_or_else(|e| { 
            eprintln!("Syntax Error: {}", e); 
            process::exit(1); 
        });

    // 4. Transform to AST and Emit SystemVerilog
    let ast = transformer::build_ast(pairs);
    let mut emitter = Emitter::new();
    emitter.emit_file(&ast);

    // 5. Write output to .sv file
    let output_path = Path::new(input_path).with_extension("sv");
    fs::write(&output_path, emitter.output)
        .unwrap_or_else(|_| { 
            eprintln!("Error: Could not write output file"); 
            process::exit(1); 
        });

    println!("Successfully transpiled '{}' to '{}'", input_path, output_path.display());
}

