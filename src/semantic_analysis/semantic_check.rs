use simpla_parser::syntax_tree::Program;

use super::body_check::{check_function_declaration, check_main_body};
use super::name_table::{name_table_factory, FactoryLocalVariableTable};
use super::semantic_error::SemanticError;
use super::variable_check::check_variables;

pub fn semantic_check<'a>(program: &'a Program) -> Result<(), SemanticError<'a>> {
    let table_factory = init_table(&program)?;

    for decl in &program.functions {
        let mut local_table = table_factory.factory_local_table();
        check_function_declaration(decl, &mut local_table);
    }

    check_main_body(&program.body, &table_factory.factory_local_table());

    Ok(())
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
