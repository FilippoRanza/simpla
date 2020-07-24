use std::collections::HashSet;

use super::name_table::LocalVariableTable;
use super::semantic_error::{ForLoopError, MismatchedAssignment, SemanticError, NonBooleanCondition, ReturnError};
use super::type_check::{function_call_check, type_check};
use simpla_parser::syntax_tree;

pub enum Contex<'a> {
    Global,
    Function(&'a syntax_tree::FuncDecl),
}

pub fn statement_check<'a>(
    stat: &'a syntax_tree::Stat,
    table: &'a LocalVariableTable,
    contex: &Contex,
) -> Result<(), SemanticError<'a>> {
    let mut loop_contex = LoopContext::new();
    stat_check(stat, table, contex, &mut loop_contex)
}

fn stat_check<'b, 'a: 'b>(
    stat: &'a syntax_tree::Stat,
    table: &'a LocalVariableTable,
    contex: &Contex,
    loop_contex: &mut LoopContext<'b>,
) -> Result<(), SemanticError<'a>> {
    match stat {
        syntax_tree::Stat::AssignStat(assign_stat) => {
            check_assign_stat(assign_stat, table, loop_contex)
        }
        syntax_tree::Stat::Break => check_break_stat(loop_contex),
        syntax_tree::Stat::ForStat(for_stat) => {
            check_for_stat(for_stat, table, contex, loop_contex)
        }
        syntax_tree::Stat::FuncCall(func_call) => function_call_check(func_call, table),
        syntax_tree::Stat::IfStat(if_stat) => check_if_stat(if_stat, table, contex, loop_contex),
        syntax_tree::Stat::ReadStat(read_stat) => check_read_stat(read_stat, table),
        syntax_tree::Stat::ReturnStat(return_stat) => check_return_stat(return_stat, table, contex),
        syntax_tree::Stat::WhileStat(while_stat) => {
            check_while_stat(while_stat, table, contex, loop_contex)
        }
        syntax_tree::Stat::WriteStat(write_stat) => {
            check_write_stat(write_stat, table)
        }
    }
}

fn stat_list_check<'b, 'a: 'b>(
    stat_list: &'a syntax_tree::StatList,
    table: &'a LocalVariableTable,
    contex: &Contex,
    loop_contex: &mut LoopContext<'b>,
) -> Result<(), SemanticError<'a>> {
    for stat in stat_list.iter() {
        stat_check(stat, table, contex, loop_contex)?;
    }
    Ok(())
}

enum CheckStatus {
    Success,
    Failure,
}

struct LoopContext<'a> {
    indexes: HashSet<&'a str>,
    nested_loops: usize,
}

impl<'a> LoopContext<'a> {
    fn new() -> Self {
        Self {
            indexes: HashSet::new(),
            nested_loops: 0,
        }
    }

    fn enter_for_loop(&mut self, index: &'a str) -> CheckStatus {
        if self.indexes.contains(index) {
            CheckStatus::Failure
        } else {
            self.nested_loops += 1;
            self.indexes.insert(index);
            CheckStatus::Success
        }
    }

    fn exit_for_loop(&mut self, index: &'a str) {
        self.nested_loops -= 1;
        self.indexes.remove(index);
    }

    fn enter_while_loop(&mut self) {
        self.nested_loops += 1;
    }

    fn exit_while_loop(&mut self) {
        self.nested_loops -= 1;
    }

    fn check_assign(&self, var_name: &'a str) -> CheckStatus {
        if self.nested_loops > 0 && self.indexes.contains(var_name) {
            CheckStatus::Failure
        } else {
            CheckStatus::Success
        }
    }

    fn check_break(&self) -> CheckStatus {
        if self.nested_loops == 0 {
            CheckStatus::Failure
        } else {
            CheckStatus::Success
        }
    }
}

fn check_assign_stat<'b, 'a: 'b>(
    assign_stat: &'a syntax_tree::AssignStat,
    table: &'a LocalVariableTable,
    contex: &LoopContext<'b>,
) -> Result<(), SemanticError<'a>> {
    let right_kind = type_check(&assign_stat.expr, table)?;
    let left_kind = table.get_variable(&assign_stat.id)?;
    if left_kind == &right_kind {
        match contex.check_assign(&assign_stat.id) {
            CheckStatus::Success => Ok(()),
            CheckStatus::Failure => {
                let err = ForLoopError::CountVariableAssignment(&assign_stat.id);
                Err(SemanticError::ForLoopError(err))
            }
        }
    } else {
        let err = MismatchedAssignment::new(&assign_stat.id, left_kind.clone(), right_kind);
        Err(SemanticError::MismatchedAssignment(err))
    }
}

fn check_break_stat<'a>(loop_contex: &LoopContext) -> Result<(), SemanticError<'a>> {
    match loop_contex.check_break() {
        CheckStatus::Success => Ok(()),
        CheckStatus::Failure => Err(SemanticError::BreakOutsideLoop),
    }
}

fn check_for_stat<'b, 'a: 'b>(
    for_stat: &'a syntax_tree::ForStat,
    table: &'a LocalVariableTable,
    block_contex: &Contex,
    loop_contex: &mut LoopContext<'b>,
) -> Result<(), SemanticError<'a>> {
    match table.get_variable(&for_stat.id)? {
        syntax_tree::Kind::Int => {}
        other => {
            let err = ForLoopError::NonIntegerCount(other.clone());
            return Err(SemanticError::ForLoopError(err));
        }
    }

    match type_check(&for_stat.begin_expr, table)? {
        syntax_tree::Kind::Int => {}
        other => {
            let err = ForLoopError::NonIntegerStart(other.clone());
            return Err(SemanticError::ForLoopError(err));
        }
    }

    match type_check(&for_stat.end_expr, table)? {
        syntax_tree::Kind::Int => {}
        other => {
            let err = ForLoopError::NonIntegerEnd(other.clone());
            return Err(SemanticError::ForLoopError(err));
        }
    }

    match loop_contex.enter_for_loop(&for_stat.id) {
        CheckStatus::Success => {
            stat_list_check(&for_stat.body, table, block_contex, loop_contex)?;
            loop_contex.exit_for_loop(&for_stat.id);
            Ok(())
        }
        CheckStatus::Failure => {
            let err = ForLoopError::CountVariableAssignment(&for_stat.id);
            Err(SemanticError::ForLoopError(err))
        }
    }
}

fn check_if_stat<'b, 'a: 'b>(
    if_stat: &'a syntax_tree::IfStat,
    table: &'a LocalVariableTable,
    block_contex: &Contex,
    loop_contex: &mut LoopContext<'b>,
) -> Result<(), SemanticError<'a>> {
    
    match type_check(&if_stat.cond, table)? {
        syntax_tree::Kind::Bool => {
            stat_list_check(&if_stat.if_body, table, block_contex, loop_contex)?;
            if let Some(ref stat_list) = if_stat.else_body {
                stat_list_check(stat_list, table, block_contex, loop_contex)?;
            }
            Ok(())
        },
        other => {
            let err = NonBooleanCondition::IfStat(other.clone());
            Err(SemanticError::NonBooleanCondition(err))
        }
    }

}

fn check_read_stat<'a>(
    read_stat: &'a syntax_tree::IdList,
    table: &'a LocalVariableTable,
) -> Result<(), SemanticError<'a>> {
    for id in read_stat {
        table.get_variable(id)?;
    }
    Ok(())
}

fn check_return_stat<'a>(
    return_stat: &'a Option<syntax_tree::Expr>,
    table: &'a LocalVariableTable,
    block_contex: &Contex,
) -> Result<(), SemanticError<'a>> {
    match block_contex {
        Contex::Function(func_decl) => {
            match &func_decl.kind {
                syntax_tree::Kind::Void => {
                    match return_stat {
                        Some(stat) => {
                            let kind = type_check(stat, table)?;
                            let err = ReturnError::MismatchedReturnType(syntax_tree::Kind::Void, kind);
                            Err(SemanticError::ReturnError(err))
                        }, 
                        None => Ok(())
                    }
                },
                other => {
                    match return_stat {
                        Some(stat) => {
                            let kind = type_check(stat, table)?;
                            if &kind == other {
                                Ok(())
                            } else {
                                let err = ReturnError::MismatchedReturnType(other.clone(), kind);
                                Err(SemanticError::ReturnError(err))
                            }
                        }
                        None => {
                            let err = ReturnError::MismatchedReturnType(other.clone(), syntax_tree::Kind::Void);
                            Err(SemanticError::ReturnError(err))
                        }
                    }
                }
            }
        },
        Contex::Global => {
            let err = ReturnError::ReturnOutsideFunction;
            Err(SemanticError::ReturnError(err))
        }
    }
}

fn check_while_stat<'b, 'a: 'b>(
    while_stat: &'a syntax_tree::WhileStat,
    table: &'a LocalVariableTable,
    block_contex: &Contex,
    loop_contex: &mut LoopContext<'b>,
) -> Result<(), SemanticError<'a>> {

    match type_check(&while_stat.cond, table)? {
        syntax_tree::Kind::Bool => {
            loop_contex.enter_while_loop();
            stat_list_check(&while_stat.body, table, block_contex, loop_contex)?;
            loop_contex.exit_while_loop();
            Ok(())
        },
        other => {
            let err = NonBooleanCondition::WhileStat(other.clone());
            Err(SemanticError::NonBooleanCondition(err))
        }
    }
}

fn check_write_stat<'a>(
    write_stat: &'a syntax_tree::WriteStat,
    table: &'a LocalVariableTable,
) -> Result<(), SemanticError<'a>> {
    match write_stat {
        syntax_tree::WriteStat::Write(expr_list) => check_expr_list(expr_list, table),
        syntax_tree::WriteStat::WriteLine(expr_list) => check_expr_list(expr_list, table)
    }
    
}

fn check_expr_list<'a>(list: &'a syntax_tree::ExprList, table: &'a LocalVariableTable) -> Result<(), SemanticError<'a>> {
    for expr in list {
        type_check(expr, table)?;
    }
    Ok(())
}
