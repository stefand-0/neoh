use pest::Parser;
use pest_derive::Parser;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar_inline = r#"
// Core File structure
file = { SOI ~ (known_stmt | piece_def | block_def | testbench | testgroup | aff_stmt)* ~ EOI }

// Global Utilities
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT    = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" | "//" ~ (!"\n" ~ ANY)* ~ "\n" }

identifier = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }
int        = @{ ASCII_DIGIT+ }
string_literal = @{ "\"" ~ (!"\"" ~ ANY)* ~ "\"" }

// 1. Macros ("known")
known_stmt = { "known" ~ identifier ~ "<=" ~ identifier ~ ("," ~ identifier)* ~ ";"? }

// 2. Pieces (Interfaces)
piece_def  = { "piece" ~ identifier ~ "{" ~ piece_port* ~ "}" }
piece_port = { ("in" | "out") ~ identifier ~ ","? }

// 3. Modular Blocks
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

// 4. Testbenches (With target(...) and Block/Verification statements)
testbench   = { "testbench" ~ identifier ~ "target" ~ "(" ~ identifier ~ ")" ~ "{" ~ (block_stmt | verif_cmd)* ~ "}" }
verif_cmd   = { getvars_cmd | when_cmd | out_cmd | put_cmd | expect_cmd | pulse_cmd | watch_cmd | writefile_cmd }
getvars_cmd = { "getvars" ~ "(" ~ getvar_item ~ ("," ~ getvar_item)* ~ ")" ~ ";" }
getvar_item = { "!"? ~ (identifier | "*") }
when_cmd    = { "when" ~ "(" ~ identifier ~ ")" ~ "{" ~ verif_cmd* ~ "}" }

// Exact syntax updates
out_cmd       = { "/" ~ int ~ "out" ~ "(" ~ (string_literal | identifier) ~ ")" ~ ";"? }
put_cmd       = { "put" ~ identifier ~ "<=" ~ expression ~ ";" }
expect_cmd    = { "/" ~ int ~ "expect" ~ "(" ~ expression ~ ")" ~ ";" }
pulse_cmd     = { "pulse" ~ "len" ~ "(" ~ identifier ~ ")" ~ "," ~ "gap" ~ "(" ~ identifier ~ ")" ~ ";" }
watch_cmd     = { "/" ~ int ~ "watchfor" ~ identifier ~ "<=" ~ identifier ~ "&" ~ out_cmd }
writefile_cmd = { "writefile" ~ "(" ~ "mode" ~ identifier ~ "," ~ "file" ~ identifier ~ "." ~ identifier ~ ")" ~ ";" }

// 5. Testgroups
testgroup      = { "testgroup" ~ identifier ~ "{" ~ testgroup_item* ~ "}" }
testgroup_item = { do_cmd | same_block }
do_cmd         = { "do" ~ identifier ~ ";" }
same_block     = { "same" ~ "{" ~ identifier* ~ "}" }

// 6. Always_ff Blocks ("aff")
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
    let output_path = if args.len() >= 4 && args[2] == "-o" {
        args[3].clone()
    } else {
        Path::new(input_path).with_extension("sv").to_string_lossy().into_owned()
    };

    let content = fs::read_to_string(input_path).expect("Failed to read input file");

    match NeoHParser::parse(Rule::file, &content) {
        Ok(mut pairs) => {
            let file_pair = pairs.next().unwrap();
            let mut sv_output = String::new();
            sv_output.push_str("// Transpiled from NeoH to SystemVerilog\n\n");

            for pair in file_pair.into_inner() {
                match pair.as_rule() {
                    Rule::piece_def => sv_output.push_str(&transpile_piece(pair)),
                    Rule::block_def => sv_output.push_str(&transpile_block(pair)),
                    Rule::testbench => sv_output.push_str(&transpile_testbench(pair)),
                    Rule::testgroup => sv_output.push_str(&transpile_testgroup(pair)),
                    Rule::aff_stmt => sv_output.push_str(&transpile_aff(pair)),
                    Rule::known_stmt => sv_output.push_str(&format!("// Macro Definition: {}\n\n", pair.as_str().trim())),
                    _ => {}
                }
            }

            fs::write(&output_path, sv_output).expect("Failed to write SystemVerilog output file");
            println!("Compilation successful! Written to {}", output_path);
        }
        Err(e) => {
            eprintln!("Syntax Error: {}", e);
            std::process::exit(1);
        }
    }
}

// --- Transpilation Emitters ---

fn transpile_piece(pair: pest::iterators::Pair<Rule>) -> String {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str();
    let mut sv = format!("interface {};\n", name);
    for port in inner {
        let mut p_inner = port.into_inner();
        let dir = p_inner.next().unwrap().as_str();
        let port_name = p_inner.next().unwrap().as_str();
        sv.push_str(&format!("  logic {}; // {}\n", port_name, dir));
    }
    sv.push_str("endinterface\n\n");
    sv
}

fn transpile_block(pair: pest::iterators::Pair<Rule>) -> String {
    let _is_pieced = pair.as_str().starts_with("pieced");
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str();
    
    let mut sv = format!("module {}(\n", name);
    let mut stmts = Vec::new();
    
    for item in inner {
        match item.as_rule() {
            Rule::port_list => {
                for p in item.into_inner() {
                    if p.as_rule() == Rule::port {
                        let mut p_in = p.into_inner();
                        let dir = match p_in.next().unwrap().as_str() {
                            "in" => "input",
                            _ => "output"
                        };
                        let p_name = p_in.next().unwrap().as_str();
                        sv.push_str(&format!("  {} logic {},\n", dir, p_name));
                    }
                }
            },
            Rule::block_stmt => stmts.push(item),
            _ => {}
        }
    }
    if sv.ends_with(",\n") { sv.pop(); sv.pop(); sv.push_str("\n"); }
    sv.push_str(");\n\n");

    for stmt in stmts {
        sv.push_str(&transpile_block_statement(stmt, "  "));
    }

    sv.push_str("endmodule\n\n");
    sv
}

fn transpile_block_statement(pair: pest::iterators::Pair<Rule>, indent: &str) -> String {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::var_decl => {
            let id = inner.into_inner().next().unwrap().as_str();
            format!("{}logic {};\n", indent, id)
        }
        Rule::assign_stmt => {
            let mut i = inner.into_inner();
            let dest = i.next().unwrap().as_str();
            let expr = i.next().unwrap().as_str();
            format!("{}assign {} = {};\n", indent, dest, expr)
        }
        Rule::ret_stmt => {
            let mut i = inner.into_inner();
            let id = i.next().unwrap().as_str();
            let expr = i.last().unwrap().as_str(); 
            format!("{}assign {} = {};\n", indent, id, expr)
        }
        Rule::pass_params => {
            let mut i = inner.into_inner();
            let inst = i.next().unwrap().as_str();
            let mod_name = i.next().unwrap().as_str();
            format!("{}{} {}();\n", indent, mod_name, inst)
        }
        _ => format!("{}// Unhandled statement\n", indent)
    }
}

fn transpile_testbench(pair: pest::iterators::Pair<Rule>) -> String {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str();
    let target = inner.next().unwrap().as_str();

    let mut sv = format!("module {}_tb;\n", name);
    sv.push_str(&format!("  // Target Instance\n  {} uut();\n\n", target));

    let mut inside_initial = false;
    
    for item in inner {
        match item.as_rule() {
            Rule::block_stmt => {
                sv.push_str(&transpile_block_statement(item, "  "));
            }
            Rule::verif_cmd => {
                let sub = item.into_inner().next().unwrap();
                if sub.as_rule() == Rule::when_cmd {
                    if !inside_initial {
                        sv.push_str("  initial begin\n");
                        inside_initial = true;
                    }
                    for cmd in sub.into_inner().skip(1) {
                        sv.push_str(&transpile_verif_command(cmd, "    "));
                    }
                } else {
                    if !inside_initial {
                        sv.push_str("  initial begin\n");
                        inside_initial = true;
                    }
                    sv.push_str(&transpile_verif_command(sub, "    "));
                }
            }
            _ => {}
        }
    }

    if inside_initial {
        sv.push_str("  end\n");
    }
    sv.push_str("endmodule\n\n");
    sv
}

fn transpile_verif_command(pair: pest::iterators::Pair<Rule>, indent: &str) -> String {
    match pair.as_rule() {
        Rule::out_cmd => {
            let mut i = pair.into_inner();
            let delay = i.next().unwrap().as_str();
            let msg = i.next().unwrap().as_str();
            format!("{}#{} $display({});\n", indent, delay, msg)
        }
        Rule::put_cmd => {
            let mut i = pair.into_inner();
            let id = i.next().unwrap().as_str();
            let expr = i.next().unwrap().as_str();
            format!("{}{} <= {};\n", indent, id, expr)
        }
        Rule::expect_cmd => {
            let mut i = pair.into_inner();
            let delay = i.next().unwrap().as_str();
            let expr = i.next().unwrap().as_str();
            format!("{}#{} assert({});\n", indent, delay, expr)
        }
        Rule::pulse_cmd => {
            let mut i = pair.into_inner();
            let len = i.next().unwrap().as_str();
            let gap = i.next().unwrap().as_str();
            format!("{}// Pulse logic: len {}, gap {}\n", indent, len, gap)
        }
        Rule::writefile_cmd => {
            format!("{}// VCD Dumping Setup\n", indent)
        }
        _ => format!("{}// {}\n", indent, pair.as_str().trim())
    }
}

fn transpile_testgroup(pair: pest::iterators::Pair<Rule>) -> String {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str();
    let mut sv = format!("module {};\n  initial begin\n", name);

    for item in inner {
        match item.as_rule() {
            Rule::testgroup_item => {
                let sub = item.into_inner().next().unwrap();
                match sub.as_rule() {
                    Rule::do_cmd => {
                        let tb = sub.into_inner().next().unwrap().as_str();
                        sv.push_str(&format!("    $display(\"Executing Testbench: {}\");\n", tb));
                    }
                    Rule::same_block => {
                        sv.push_str("    fork\n");
                        for id in sub.into_inner() {
                            sv.push_str(&format!("      $display(\"Concurrent running: {}\");\n", id.as_str()));
                        }
                        sv.push_str("    join\n");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    sv.push_str("  end\nendmodule\n\n");
    sv
}

fn transpile_aff(pair: pest::iterators::Pair<Rule>) -> String {
    let mut inner = pair.into_inner();
    let edge_expr = inner.next().unwrap().as_str();
    let mut sv = format!("always_ff @({}) begin\n", edge_expr);
    for stmt in inner {
        sv.push_str(&transpile_block_statement(stmt, "  "));
    }
    sv.push_str("end\n\n");
    sv
}
