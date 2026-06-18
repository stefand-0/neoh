use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct NeoParser;
// that's it.
