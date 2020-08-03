use structopt::StructOpt;
mod semantic_analysis;
use simpla_parser;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

#[derive(StructOpt)]
struct Arguments {
    source_file: PathBuf,
}

fn load_file(file: &Path) -> std::io::Result<String> {

    let mut file = File::open(file)?;
    let mut output = String::new();
    file.read_to_string(&mut output)?;

    Ok(output)

}

fn compile(code: &str) -> Result<(), String> {
    let parser = simpla_parser::ProgramParser::new();
    let program = parser.parse(code).unwrap();
    semantic_analysis::semantic_check(&program, code)
}

fn main() {
    let args = Arguments::from_args();
    let prog = load_file(&args.source_file).unwrap();
    match compile(&prog) {
        Ok(()) => {}
        Err(err_msg) => eprintln!("{}", err_msg),
    }
}
