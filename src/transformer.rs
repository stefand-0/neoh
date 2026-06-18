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

pub fn build_put(pair: pest::iterators::Pair<Rule>) -> PutStmt {
    let mut inner = pair.into_inner();
    let target = inner.next().unwrap().as_str().to_string();
    let op = inner.next().unwrap().as_str().to_string();
    let expr = inner.next().unwrap().as_str().to_string();
    let width = inner.find_map(|p| if p.as_rule() == Rule::width_constraint { Some(parse_width(p)) } else { None });
    PutStmt { target, op, expr, width }
}

fn parse_width(pair: pest::iterators::Pair<Rule>) -> WidthConstraint {
    let mut inner = pair.into_inner();
    let msb = inner.next().unwrap().as_str().parse().unwrap();
    let lsb = inner.next().unwrap().as_str().parse().unwrap();
    WidthConstraint { msb, lsb }
}
