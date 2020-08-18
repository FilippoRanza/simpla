use structopt::StructOpt;
mod generator;
mod semantic_analysis;
use simpla_parser;
use simpla_parser::syntax_tree;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(StructOpt, Debug)]
enum Action {
    #[structopt(about = "Check for program correctness")]
    Check { source_file: PathBuf },
    #[structopt(about = "Compile a Simpla program to simplac bytecode")]
    Translate(Arguments),
    #[structopt(about = "Compile a Simpla program to C source file")]
    Compile(Arguments),
}

#[derive(StructOpt, Debug)]
struct Arguments {
    source_file: PathBuf,
    output_file: Option<PathBuf>,
}

fn load_file(file: &Path) -> std::io::Result<String> {
    let mut file = File::open(file)?;
    let mut output = String::new();
    file.read_to_string(&mut output)?;

    Ok(output)
}

fn get_file_name(arg: Arguments) -> PathBuf {
    if let Some(output) = arg.output_file {
        output
    } else {
        let input_name = arg.source_file;
        let str_name = input_name.to_str().unwrap();
        let i = str_name.find('.');
        if let Some(i) = i {
            let without_extension = &str_name[..i];
            let file_name = format!("{}.simplac", without_extension);
            Path::new(&file_name).to_path_buf()
        } else {
            input_name
        }
    }
}

fn save_to_file(arg: Arguments, code: Vec<u8>) -> std::io::Result<()> {
    let file_name = get_file_name(arg);
    let mut output_file = File::create(&file_name)?;
    output_file.write(&code)?;
    Ok(())
}

fn generate_ast(code: &str) -> Result<syntax_tree::Program, String> {
    let parser = simpla_parser::ProgramParser::new();
    let program = parser.parse(code).unwrap();
    semantic_analysis::semantic_check(&program, code)?;
    Ok(program)
}

fn compile_to_ast(path: &Path) -> Result<syntax_tree::Program, String> {
    match load_file(path) {
        Ok(code) => generate_ast(&code),
        Err(err) => Err(format!("{}", err)),
    }
}

fn compile<Func>(arg: Arguments, translator: Func) -> Result<(), String>
where
    Func: Fn(&syntax_tree::Program) -> Vec<u8>,
{
    let ast = compile_to_ast(&arg.source_file)?;
    let code = translator(&ast);
    match save_to_file(arg, code) {
        Ok(()) => Ok(()),
        Err(err) => Err(format!("{}", err)),
    }
}

fn run_program(action: Action) -> Result<(), String> {
    match action {
        Action::Check { source_file } => {
            let _ = compile_to_ast(&source_file)?;
        }
        Action::Compile(arg) => compile(arg, generator::translate_to_c)?,
        Action::Translate(_arg) => panic!("Not implemented"),
    }

    Ok(())
}

fn main() {
    let args = Action::from_args();

    match run_program(args) {
        Ok(()) => {}
        Err(msg) => eprintln!("{}", msg),
    }

    /*
    let prog = load_file(&args.source_file).unwrap();
    match compile(&prog) {
        Ok(byte_code) => {
            let res = save_to_file(args, byte_code);
            match res {
                Ok(()) => {}
                Err(io_err) => eprintln!("{}", io_err),
            };
        }
        Err(err_msg) => eprintln!("{}", err_msg),
    }
    */
}
