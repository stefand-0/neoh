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


use std::env;
use std::fs;
use std::process;
use pest::Parser;

mod ast;
mod emitter;
mod parser;
mod transformer;

use crate::parser::{NeoParser, Rule};
use crate::emitter::Emitter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { process::exit(1); }
    let source = fs::read_to_string(&args[1]).unwrap();
    let pairs = NeoParser::parse(Rule::file, &source).unwrap();
    let ast = transformer::build_ast(pairs);
    let mut emitter = Emitter::new();
    emitter.emit_file(&ast);
    fs::write(args[1].replace(".neoh", ".sv"), emitter.output).unwrap();
}
