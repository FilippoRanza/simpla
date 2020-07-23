use super::name_table::{LocalVariableTable, LookupTable};
use super::semantic_error::{MismatchedAssignment, SemanticError};
use super::type_check::type_check;
use simpla_parser::syntax_tree;

pub fn check_function_declaration<'a>(
    func_decl: &'a syntax_tree::FuncDecl,
    table: &'a LocalVariableTable,
) -> Result<(), SemanticError<'a>> {
    Ok(())
}

pub fn check_main_body<'a>(
    stat_list: &'a syntax_tree::StatList,
    table: &'a LocalVariableTable,
) -> Result<(), SemanticError<'a>> {
    Ok(())
}

enum Contex<'a> {
    Global,
    Function(&'a syntax_tree::FuncDecl),
    Loop,
    FunctionLoop(&'a syntax_tree::FuncDecl),
}

fn stat_check<'a>(
    stat: &'a syntax_tree::Stat,
    table: &'a LocalVariableTable,
) -> Result<(), SemanticError<'a>> {
    match stat {
        syntax_tree::Stat::AssignStat(assign_stat) =>
            /*check_assign_stat(assign_stat, table)*/
            {}
        syntax_tree::Stat::Break => {}
        syntax_tree::Stat::ForStat(for_stat) => {}
        syntax_tree::Stat::FuncCall(func_call) => {}
        syntax_tree::Stat::IfStat(if_stat) => {}
        syntax_tree::Stat::ReadStat(read_stat) => {}
        syntax_tree::Stat::ReturnStat(return_stat) => {}
        syntax_tree::Stat::WhileStat(while_stat) => {}
        syntax_tree::Stat::WriteStat(write_stat) => {}
    }
    Ok(())
}

fn check_assign_stat<'a>(
    assign_stat: &'a syntax_tree::AssignStat,
    table: &'a LookupTable,
) -> Result<(), SemanticError<'a>> {
    let right_kind = type_check(&assign_stat.expr, table)?;
    let left_kind = table.get_variable(&assign_stat.id)?;
    if left_kind == &right_kind {
        Ok(())
    } else {
        let err = MismatchedAssignment::new(&assign_stat.id, left_kind.clone(), right_kind);
        Err(SemanticError::MismatchedAssignment(err))
    }
}
