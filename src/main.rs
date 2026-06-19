use pest::Parser;
use pest_derive::Parser;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar_inline = r#"
file = { SOI ~ (known_stmt | piece_def | block_def | testbench | testgroup | aff_stmt)* ~ EOI }
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT    = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" | "//" ~ (!"\n" ~ ANY)* ~ "\n" }

identifier = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }
int        = @{ ASCII_DIGIT+ }
string_literal = @{ "\"" ~ (!"\"" ~ ANY)* ~ "\"" }

known_stmt = { "known" ~ identifier ~ "<=" ~ identifier ~ ("," ~ identifier)* ~ ";"? }
piece_def  = { "piece" ~ identifier ~ "{" ~ piece_port* ~ "}" }
piece_port = { ("in" | "out") ~ identifier ~ ","? }

block_def  = { "pieced"? ~ "block" ~ identifier ~ "(" ~ port_list? ~ ")" ~ identifier? ~ "{" ~ block_stmt* ~ "}" }
port_list  = { (port | piece_bind) ~ ("," ~ (port | piece_bind))* }
port       = { ("in" | "out") ~ identifier }
piece_bind = { identifier ~ "<=" ~ identifier }

block_stmt  = { ret_stmt | pass_params | var_decl | assign_stmt }
var_decl    = { "var" ~ identifier ~ ";" }
assign_stmt = { identifier ~ "=" ~ expression ~ ";" }
pass_params = { identifier ~ "passparams" ~ identifier ~ "(" ~ identifier ~ ("," ~ identifier)* ~ ")" ~ ";" }
ret_stmt    = { "ret" ~ "tempassign"? ~ identifier ~ ("[" ~ "always" ~ int ~ ":" ~ int ~ "]")? ~ "<=" ~ expression ~ ";" }
expression  = { operand ~ (operator ~ operand)* }
operand     = { string_literal | identifier | int }
operator    = { "+" | "-" | "&" | "|" | "==" | "<=" }

testbench   = { "testbench" ~ identifier ~ "target" ~ "(" ~ identifier ~ ")" ~ "{" ~ (block_stmt | verif_cmd)* ~ "}" }
verif_cmd   = { getvars_cmd | when_cmd | out_cmd | put_cmd | expect_cmd | pulse_cmd | watch_cmd | writefile_cmd }
getvars_cmd = { "getvars" ~ "(" ~ getvar_item ~ ("," ~ getvar_item)* ~ ")" ~ ";" }
getvar_item = { "!"? ~ (identifier | "*") }
when_cmd    = { "when" ~ "(" ~ identifier ~ ")" ~ "{" ~ verif_cmd* ~ "}" }

out_cmd       = { "/" ~ int ~ "out" ~ "(" ~ (string_literal | identifier) ~ ")" ~ ";"? }
put_cmd       = { "put" ~ identifier ~ "<=" ~ expression ~ ";" }
expect_cmd    = { "/" ~ int ~ "expect" ~ "(" ~ expression ~ ")" ~ ";" }
pulse_cmd     = { "pulse" ~ "len" ~ "(" ~ identifier ~ ")" ~ "," ~ "gap" ~ "(" ~ identifier ~ ")" ~ ";" }
watch_cmd     = { "/" ~ int ~ "watchfor" ~ identifier ~ "<=" ~ identifier ~ "&" ~ out_cmd }
writefile_cmd = { "writefile" ~ "(" ~ "mode" ~ identifier ~ "," ~ "file" ~ identifier ~ "." ~ identifier ~ ")" ~ ";" }

testgroup      = { "testgroup" ~ identifier ~ "{" ~ testgroup_item* ~ "}" }
testgroup_item = { do_cmd | same_block }
do_cmd         = { "do" ~ identifier ~ ";" }
same_block     = { "same" ~ "{" ~ identifier* ~ "}" }

aff_stmt  = { "aff" ~ edge_expr ~ "{" ~ block_stmt* ~ "}" }
edge_expr = { edge_term ~ ("or" ~ edge_term)* }
edge_term = { ("posedge" | "negedge") ~ "(" ~ identifier ~ ")" }
"#]
struct NeoHParser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: neoh <input_file.neoh> [-o <output_file.sv>]");
        std::process::exit(1);
    }
    let input_path = &args[1];
    let output_path = if args.len() >= 4 && args[2] == "-o" { args[3].clone() } else { Path::new(input_path).with_extension("sv").to_string_lossy().into_owned() };
    let content = fs::read_to_string(input_path).expect("Failed to read input file");

    match NeoHParser::parse(Rule::file, &content) {
        Ok(mut pairs) => {
            if let Some(file_pair) = pairs.next() {
                let mut sv_output = String::new();
                for pair in file_pair.into_inner() {
                    match pair.as_rule() {
                        Rule::piece_def => sv_output.push_str(&transpile_piece(pair)),
                        Rule::block_def => sv_output.push_str(&transpile_block(pair)),
                        Rule::testbench => sv_output.push_str(&transpile_testbench(pair)),
                        Rule::testgroup => sv_output.push_str(&transpile_testgroup(pair)),
                        Rule::aff_stmt => sv_output.push_str(&transpile_aff(pair)),
                        _ => {}
                    }
                }
                fs::write(&output_path, sv_output).expect("Failed to write output");
                println!("Success: {}", output_path);
            }
        }
        Err(e) => eprintln!("Syntax Error: {}", e),
    }
}

fn transpile_piece(pair: pest::iterators::Pair<Rule>) -> String {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str()).unwrap_or("UnknownPiece");
    let mut sv = format!("interface {};\n", name);
    for port in inner {
        if port.as_rule() == Rule::piece_port {
            let port_str = port.as_str().trim();
            let dir = if port_str.starts_with("in") { "in" } else { "out" };
            if let Some(n) = port.into_inner().next() {
                sv.push_str(&format!("  logic {}; // {}\n", n.as_str(), dir));
            }
        }
    }
    sv.push_str("endinterface\n\n");
    sv
}

fn find_ports(pair: pest::iterators::Pair<Rule>, ports: &mut Vec<String>) {
    if pair.as_rule() == Rule::port {
        let port_str = pair.as_str().trim();
        let dir = if port_str.starts_with("in") { "input" } else { "output" };
        if let Some(n) = pair.into_inner().next() {
            ports.push(format!("{} logic {}", dir, n.as_str()));
        }
    } else {
        for child in pair.into_inner() {
            find_ports(child, ports);
        }
    }
}

fn transpile_block(pair: pest::iterators::Pair<Rule>) -> String {
    let mut ports = Vec::new();
    find_ports(pair.clone(), &mut ports);

    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str()).unwrap_or("UnknownBlock");
    let mut stmts = Vec::new();
    for item in inner {
        if item.as_rule() == Rule::block_stmt { stmts.push(item); }
    }
    let mut sv = format!("module {}(\n  {}\n);\n\n", name, ports.join(",\n  "));
    for stmt in stmts { sv.push_str(&transpile_block_statement(stmt, "  ")); }
    sv.push_str("endmodule\n\n");
    sv
}

fn transpile_block_statement(pair: pest::iterators::Pair<Rule>, indent: &str) -> String {
    let inner = match pair.into_inner().next() { Some(i) => i, None => return String::new() };
    match inner.as_rule() {
        Rule::var_decl => format!("{}logic {};\n", indent, inner.into_inner().next().map(|p| p.as_str()).unwrap_or("")),
        Rule::assign_stmt => {
            let mut i = inner.into_inner();
            format!("{}assign {} = {};\n", indent, i.next().map(|p| p.as_str()).unwrap_or(""), i.next().map(|p| p.as_str()).unwrap_or(""))
        }
        Rule::ret_stmt => {
            let mut i = inner.into_inner();
            let id = i.next().map(|p| p.as_str()).unwrap_or(" ");
            let expr = i.last().map(|p| p.as_str()).unwrap_or("");
            format!("{}assign {} = {};\n", indent, id, expr)
        }
        Rule::pass_params => {
            let mut i = inner.into_inner();
            format!("{}{} {}();\n", indent, i.nth(1).map(|p| p.as_str()).unwrap_or("Mod"), i.next().map(|p| p.as_str()).unwrap_or("inst"))
        }
        _ => String::new()
    }
}

fn transpile_testbench(pair: pest::iterators::Pair<Rule>) -> String {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str()).unwrap_or("Test");
    let target = inner.next().map(|p| p.as_str()).unwrap_or("Target");
    let mut sv = format!("module {}_tb;\n  {} uut();\n  initial begin\n", name, target);
    for item in inner {
        if item.as_rule() == Rule::verif_cmd {
            if let Some(sub) = item.into_inner().next() {
                sv.push_str(&transpile_verif(sub, "    "));
            }
        }
    }
    sv.push_str("  end\nendmodule\n\n");
    sv
}

fn transpile_verif(pair: pest::iterators::Pair<Rule>, indent: &str) -> String {
    match pair.as_rule() {
        Rule::out_cmd => {
            let mut i = pair.into_inner();
            format!("{}#{} $display({});\n", indent, i.next().map(|p| p.as_str()).unwrap_or("0"), i.next().map(|p| p.as_str()).unwrap_or(""))
        }
        Rule::put_cmd => {
            let mut i = pair.into_inner();
            format!("{}{} <= {};\n", indent, i.next().map(|p| p.as_str()).unwrap_or(""), i.next().map(|p| p.as_str()).unwrap_or(""))
        }
        _ => format!("{}// Command\n", indent)
    }
}

fn transpile_testgroup(pair: pest::iterators::Pair<Rule>) -> String {
    let name = pair.into_inner().next().map(|p| p.as_str()).unwrap_or("Group");
    format!("module {};\n  initial $display(\"Group {}\");\nendmodule\n\n", name, name)
}

fn transpile_aff(pair: pest::iterators::Pair<Rule>) -> String {
    let mut inner = pair.into_inner();
    format!("always_ff @({}) begin end\n\n", inner.next().map(|p| p.as_str()).unwrap_or("*"))
}
