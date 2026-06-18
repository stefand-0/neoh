use crate::ast::*;
use crate::parser::Rule;
use pest::iterators::Pairs;

pub fn build_ast(pairs: Pairs<Rule>) -> File {
    let mut statements = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::block_def => statements.push(Statement::Block(build_block(pair))),
            _ => (),
        }
    }
    File { statements }
}

fn build_block(pair: pest::iterators::Pair<Rule>) -> BlockDef {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    BlockDef { name, params: vec![], body: vec![] }
}
