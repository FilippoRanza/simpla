use simpla_parser::syntax_tree::Program;

use super::name_table::{name_table_factory, LocalVariableTable, VariableTable};
use super::semantic_error::SemanticError;
use super::variable_check::check_variables;

pub fn semantic_check<'a>(program: &'a Program) -> Result<(), SemanticError> {
    let local_table = init_table(&program)?;

    Ok(())
}

fn init_table<'a>(program: &'a Program) -> Result<LocalVariableTable<'a>, SemanticError<'a>> {
    let mut glob_var_table = name_table_factory();
    check_variables(&program.global_vars, &mut glob_var_table)?;

    let mut func_tabl = glob_var_table.switch_to_function_table();

    for func_decl in &program.functions {
        func_tabl.insert_function(&func_decl.id, func_decl)?;
    }

    let local_var_table = func_tabl.switch_to_local_table();
    Ok(local_var_table)
}
