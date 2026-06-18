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
