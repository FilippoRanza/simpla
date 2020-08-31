use simpla_parser::syntax_tree::Program;

use super::body_check::{check_function_declaration, check_main_body};
use super::name_table::{name_table_factory, FactoryLocalVariableTable};
use super::semantic_error::SemanticError;
use super::variable_check::check_variables;

pub fn semantic_check<'a>(program: &'a Program, code: &'a str) -> Result<(), String> {
    let table_factory = match init_table(&program) {
        Ok(table_factory) => table_factory,
        Err(err) => return Err(err.format_error(code)),
    };

    for decl in &program.functions {
        let mut local_table = table_factory.factory_local_table();
        let stat = check_function_declaration(decl, &mut local_table);
        convert_error(stat, code)?;
    }

    let mut local_table = table_factory.factory_local_table();
    let stat = check_main_body(&program.body, &mut local_table);
    convert_error(stat, code)
}

fn init_table<'a>(
    program: &'a Program,
) -> Result<FactoryLocalVariableTable<'a>, SemanticError<'a>> {
    let mut glob_var_table = name_table_factory();
    check_variables(&program.global_vars, &mut glob_var_table)?;

    let mut func_tabl = glob_var_table.switch_to_function_table();

    for func_decl in &program.functions {
        func_tabl.insert_function(&func_decl.id, func_decl)?;
    }

    Ok(func_tabl.switch_to_local_table())
}

fn convert_error<'a>(res: Result<(), SemanticError<'a>>, code: &'a str) -> Result<(), String> {
    match res {
        Ok(()) => Ok(()),
        Err(err) => Err(err.format_error(code)),
    }
}


#[cfg(test)]
mod test {

    use super::*;
    use simpla_parser::ProgramParser;
    use std::path::{PathBuf, Path};
    use std::fs::File;
    use std::io::Read;


    #[test]
    fn test_variable_shadowing() {
        let var_shadow_code = PathBuf::from("test_code").join("variable_shadowing-correct.simpla");
        let code = load_file(&var_shadow_code);
        let parser = ProgramParser::new();

        let prog = parser.parse(&code).unwrap();

        let result = semantic_check(&prog, &code);
        if let Err(err) = result {
            assert!(false, "{}", err)
        }


    }



    fn load_file(file: &Path) -> String {
        let mut buff = String::new();
        let mut file = File::open(file).unwrap();
        file.read_to_string(&mut buff).unwrap();
        buff
    }

}


