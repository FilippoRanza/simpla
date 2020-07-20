use simpla_parser::syntax_tree::Program;

use super::name_table::{name_table_factory, LocalVariableTable};
use super::semantic_error::SemanticError;

pub fn semantic_check(program: Program) -> Result<Program, SemanticError> {
    
    let local_table = init_table(&program)?;

    Ok(program)
}


fn init_table<'a>(program: &'a Program) -> Result<LocalVariableTable<'a>, SemanticError> {
    let mut glob_var_table = name_table_factory();
    for glob_var_decl in &program.global_vars {
        for id in &glob_var_decl.id_list {
            glob_var_table.insert_variable(id, &glob_var_decl.kind)?;
        }
    }

    let mut func_tabl = glob_var_table.switch_to_function_table();
    for func_decl in &program.functions {
        func_tabl.insert_function(&func_decl.id, func_decl)?;
    }

    let local_var_table = func_tabl.switch_to_local_table();
    Ok(local_var_table)
}



