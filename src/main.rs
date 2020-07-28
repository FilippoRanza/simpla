mod semantic_analysis;

use simpla_parser;

fn compile(code: &str) -> Result<(), String> {
    let parser = simpla_parser::ProgramParser::new();
    let program = parser.parse(code).unwrap();
    semantic_analysis::semantic_check(&program)
}

fn main() {
    match compile("")  {
        Ok(()) => {},
        Err(err_msg) => eprintln!("{}", err_msg)
    }

}
