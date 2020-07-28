use std::collections::HashSet;

use super::name_table::LocalVariableTable;
use super::semantic_error::{
    ForLoopError, MismatchedAssignment, NonBooleanCondition, ReturnError, SemanticError,
};
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
        syntax_tree::Stat::WriteStat(write_stat) => check_write_stat(write_stat, table),
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
        }
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
        Contex::Function(func_decl) => match &func_decl.kind {
            syntax_tree::Kind::Void => match return_stat {
                Some(stat) => {
                    let kind = type_check(stat, table)?;
                    let err = ReturnError::MismatchedReturnType(syntax_tree::Kind::Void, kind);
                    Err(SemanticError::ReturnError(err))
                }
                None => Ok(()),
            },
            other => match return_stat {
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
                    let err =
                        ReturnError::MismatchedReturnType(other.clone(), syntax_tree::Kind::Void);
                    Err(SemanticError::ReturnError(err))
                }
            },
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
        }
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
        syntax_tree::WriteStat::WriteLine(expr_list) => check_expr_list(expr_list, table),
    }
}

fn check_expr_list<'a>(
    list: &'a syntax_tree::ExprList,
    table: &'a LocalVariableTable,
) -> Result<(), SemanticError<'a>> {
    for expr in list {
        type_check(expr, table)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use super::super::name_table::{name_table_factory, VariableTable};
    use super::*;

    #[test]
    fn test_check_assign_stat() {
        let var_name = "test";
        let stat = syntax_tree::AssignStat::new(
            var_name.to_owned(),
            syntax_tree::Expr::Factor(syntax_tree::Factor::Const(syntax_tree::Const::IntConst(7))),
        );

        let table_factory = name_table_factory()
            .switch_to_function_table()
            .switch_to_local_table();
        let mut table = table_factory.factory_local_table();
        table
            .insert_variable(var_name, &syntax_tree::Kind::Int)
            .unwrap();

        let mut loop_contex = LoopContext::new();

        check_assign_stat(&stat, &table, &loop_contex).unwrap();
        assert!(matches!(
            loop_contex.enter_for_loop(var_name),
            CheckStatus::Success
        ));

        assert!(matches!(
            check_assign_stat(&stat, &table, &loop_contex),
            Err(
                SemanticError::ForLoopError(
                    ForLoopError::CountVariableAssignment(name)))
                    if name == var_name
        ));

        loop_contex.exit_for_loop(var_name);
        check_assign_stat(&stat, &table, &loop_contex).unwrap();

        let stat = syntax_tree::AssignStat::new(
            var_name.to_owned(),
            syntax_tree::Expr::Factor(syntax_tree::Factor::Const(syntax_tree::Const::RealConst(
                7.5,
            ))),
        );

        assert!(matches!(
                check_assign_stat(&stat, &table, &loop_contex),
                Err(SemanticError::MismatchedAssignment(mistmatch))
                if mistmatch == MismatchedAssignment::new(var_name, syntax_tree::Kind::Int, syntax_tree::Kind::Real)
        ));
    }

    #[test]
    fn test_loop_contex() {
        let mut loop_contex = LoopContext::new();

        assert!(matches!(loop_contex.check_break(), CheckStatus::Failure));
        let names = ["a", "name", "var", "stuff"];
        for name in &names {
            assert!(matches!(
                loop_contex.enter_for_loop(name),
                CheckStatus::Success
            ));
            assert!(matches!(loop_contex.check_break(), CheckStatus::Success));
        }

        assert_eq!(loop_contex.nested_loops, names.len());

        assert!(matches!(loop_contex.check_break(), CheckStatus::Success));

        for name in &names {
            assert!(matches!(loop_contex.check_break(), CheckStatus::Success));
            loop_contex.exit_for_loop(name);
        }

        assert_eq!(loop_contex.nested_loops, 0);
        assert!(matches!(loop_contex.check_break(), CheckStatus::Failure));

        loop_contex.enter_while_loop();
        assert_eq!(loop_contex.nested_loops, 1);
        assert_eq!(loop_contex.indexes.len(), 0);

        loop_contex.exit_while_loop();
        assert_eq!(loop_contex.nested_loops, 0);
        assert_eq!(loop_contex.indexes.len(), 0);
    }

    #[test]
    fn test_check_return_stat() {
        let return_stat = Some(syntax_tree::Expr::Factor(syntax_tree::Factor::Const(
            syntax_tree::Const::RealConst(2.3),
        )));
        let func_decl = syntax_tree::FuncDecl::new(
            "func_name".to_owned(),
            vec![],
            syntax_tree::Kind::Real,
            vec![],
            vec![],
        );
        let table_factory = name_table_factory()
            .switch_to_function_table()
            .switch_to_local_table();

        let table = table_factory.factory_local_table();

        check_return_stat(&return_stat, &table, &Contex::Function(&func_decl)).unwrap();

        let func_decl = syntax_tree::FuncDecl::new(
            "func_name".to_owned(),
            vec![],
            syntax_tree::Kind::Int,
            vec![],
            vec![],
        );

        let stat = check_return_stat(&return_stat, &table, &Contex::Function(&func_decl));
        assert!(
            matches!(stat, Err(SemanticError::ReturnError(ReturnError::MismatchedReturnType(correct, given)))
                if correct == syntax_tree::Kind::Int && given == syntax_tree::Kind::Real
            )
        );

        let stat = check_return_stat(&return_stat, &table, &Contex::Global);
        assert!(matches!(
            stat,
            Err(SemanticError::ReturnError(
                ReturnError::ReturnOutsideFunction
            ))
        ));
    }

    #[test]
    fn test_check_for_stat() {
        let index_var = "index";
        let for_stat = make_for_stat(
            index_var,
            syntax_tree::Const::IntConst(0),
            syntax_tree::Const::IntConst(9),
        );

        let table_factory = name_table_factory()
            .switch_to_function_table()
            .switch_to_local_table();
        let mut table = table_factory.factory_local_table();
        table
            .insert_variable(index_var, &syntax_tree::Kind::Int)
            .unwrap();

        let mut loop_contex = LoopContext::new();

        check_for_stat(&for_stat, &table, &Contex::Global, &mut loop_contex).unwrap();

        assert_eq!(loop_contex.nested_loops, 0);
        assert_eq!(loop_contex.indexes.len(), 0);

        let for_stat = make_for_stat(
            index_var,
            syntax_tree::Const::RealConst(0.0),
            syntax_tree::Const::IntConst(10),
        );
        let stat = check_for_stat(&for_stat, &table, &Contex::Global, &mut loop_contex);
        assert!(
            matches!(stat, Err(SemanticError::ForLoopError(ForLoopError::NonIntegerStart(kind))) if kind == syntax_tree::Kind::Real)
        );

        let for_stat = make_for_stat(
            index_var,
            syntax_tree::Const::IntConst(0),
            syntax_tree::Const::RealConst(10.0),
        );
        let stat = check_for_stat(&for_stat, &table, &Contex::Global, &mut loop_contex);
        assert!(
            matches!(stat, Err(SemanticError::ForLoopError(ForLoopError::NonIntegerEnd(kind))) if kind == syntax_tree::Kind::Real)
        );
    }

    fn make_for_stat(
        index: &str,
        from: syntax_tree::Const,
        to: syntax_tree::Const,
    ) -> syntax_tree::ForStat {
        syntax_tree::ForStat::new(
            index.to_owned(),
            make_constant_expr(from),
            make_constant_expr(to),
            vec![],
        )
    }

    #[test]
    fn test_check_while_stat() {
        let table_factory = name_table_factory()
            .switch_to_function_table()
            .switch_to_local_table();
        let table = table_factory.factory_local_table();
        let while_stat = make_while_stat(syntax_tree::Operator::Greater);

        let mut loop_contex = LoopContext::new();

        check_while_stat(&while_stat, &table, &Contex::Global, &mut loop_contex).unwrap();

        assert_eq!(loop_contex.nested_loops, 0);
        assert_eq!(loop_contex.indexes.len(), 0);

        let while_stat = make_while_stat(syntax_tree::Operator::Add);

        let stat = check_while_stat(&while_stat, &table, &Contex::Global, &mut loop_contex);

        assert!(
            matches!(stat, Err(SemanticError::NonBooleanCondition(NonBooleanCondition::WhileStat(kind))) if kind == syntax_tree::Kind::Real)
        );

        assert_eq!(loop_contex.nested_loops, 0);
        assert_eq!(loop_contex.indexes.len(), 0);
    }

    fn make_while_stat(op: syntax_tree::Operator) -> syntax_tree::WhileStat {
        syntax_tree::WhileStat::new(
            syntax_tree::Expr::Node(
                Box::new(make_constant_expr(syntax_tree::Const::RealConst(7.8))),
                op,
                Box::new(make_constant_expr(syntax_tree::Const::RealConst(6.8))),
            ),
            vec![],
        )
    }

    fn make_constant_expr(const_val: syntax_tree::Const) -> syntax_tree::Expr {
        syntax_tree::Expr::Factor(syntax_tree::Factor::Const(const_val))
    }
}
