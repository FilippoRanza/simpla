mod semantic_analysis;

use simpla_parser;

fn main() {
    let parser = simpla_parser::ProgramParser::new();
    let program = parser.parse("").unwrap();
    semantic_analysis::semantic_check(&program).unwrap();
    println!("Hello, world!");
}
