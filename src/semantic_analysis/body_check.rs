use super::name_table::{LocalVariableTable, VariableTable};
use super::semantic_error::SemanticError;
use super::stat_check;
use simpla_parser::syntax_tree;

pub fn check_function_declaration<'a>(
    func_decl: &'a syntax_tree::FuncDecl,
    table: &'a mut LocalVariableTable<'a>,
) -> Result<(), SemanticError<'a>> {
    build_lookup_table(func_decl, table)?;
    check_stat_list(
        &func_decl.body,
        table,
        stat_check::Contex::Function(func_decl),
    )?;
    Ok(())
}

pub fn check_main_body<'a>(
    body: &'a syntax_tree::StatList,
    table: &'a LocalVariableTable<'a>,
) -> Result<(), SemanticError<'a>> {
    check_stat_list(body, &table, stat_check::Contex::Global)?;
    Ok(())
}

fn check_stat_list<'a>(
    stat_list: &'a syntax_tree::StatList,
    table: &'a LocalVariableTable,
    contex: stat_check::Contex,
) -> Result<(), SemanticError<'a>> {
    for stat in stat_list {
        stat_check::statement_check(stat, table, &contex)?;
    }
    Ok(())
}

fn build_lookup_table<'a>(
    func_decl: &'a syntax_tree::FuncDecl,
    table: &mut LocalVariableTable<'a>,
) -> Result<(), SemanticError<'a>> {
    for param in &func_decl.params {
        table.insert_variable(&param.id, &param.kind, &func_decl.loc)?;
    }

    for var_decl in &func_decl.vars {
        for var in &var_decl.id_list {
            table.insert_variable(var, &var_decl.kind, &var_decl.loc)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {}
