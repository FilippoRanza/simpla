use super::name_table::LocalVariableTable;
use super::semantic_error::{
    ArgumentCountError, CastError, IncoherentOperation, MismatchedArgumentType, MismatchedTypes,
    MismatchedUnary, NonBooleanCondition, SemanticError,
};
use simpla_parser::syntax_tree;

pub fn function_call_check<'a>(
    func_call: &'a syntax_tree::FuncCall,
    table: &'a LocalVariableTable<'a>,
) -> Result<(), SemanticError<'a>> {
    check_function_call(func_call, table)?;
    Ok(())
}

pub fn type_check<'a>(
    expr: &'a syntax_tree::Expr,
    table: &'a LocalVariableTable,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    match expr {
        syntax_tree::Expr::Node(left, op, right) => {
            let left_type = type_check(left, table)?;
            let right_type = type_check(right, table)?;
            let output = coherent_operation(left_type, op, right_type)?;
            Ok(output)
        }
        syntax_tree::Expr::Factor(fact) => check_factor(fact, table),
    }
}

enum OperatorKind {
    Numeric,
    Relational,
    Logic,
}

impl OperatorKind {
    fn from_operator(op: &syntax_tree::Operator) -> Self {
        match op {
            syntax_tree::Operator::And | syntax_tree::Operator::Or => Self::Logic,
            syntax_tree::Operator::Add
            | syntax_tree::Operator::Sub
            | syntax_tree::Operator::Mul
            | syntax_tree::Operator::Div => Self::Numeric,
            _ => Self::Relational,
        }
    }
}

fn coherent_operation<'a>(
    left: syntax_tree::Kind,
    op: &'a syntax_tree::Operator,
    right: syntax_tree::Kind,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    if left == right {
        let op_kind = OperatorKind::from_operator(op);
        match op_kind {
            OperatorKind::Logic => match left {
                syntax_tree::Kind::Bool => Ok(left),
                _ => {
                    let err = IncoherentOperation::new(left, op.clone());
                    Err(SemanticError::IncoherentOperation(err))
                }
            },
            OperatorKind::Numeric => match left {
                syntax_tree::Kind::Int | syntax_tree::Kind::Real => Ok(left),
                _ => {
                    let err = IncoherentOperation::new(left, op.clone());
                    Err(SemanticError::IncoherentOperation(err))
                }
            },
            OperatorKind::Relational => match left {
                syntax_tree::Kind::Int | syntax_tree::Kind::Real => Ok(syntax_tree::Kind::Bool),
                _ => {
                    let err = IncoherentOperation::new(left, op.clone());
                    Err(SemanticError::IncoherentOperation(err))
                }
            },
        }
    } else {
        let mismatch = MismatchedTypes::new(left, right);
        Err(SemanticError::MismatchedOperationTypes(mismatch))
    }
}

fn check_factor<'a>(
    fact: &'a syntax_tree::Factor,
    table: &'a LocalVariableTable,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    match fact {
        syntax_tree::Factor::CastExpr(cast) => check_cast(cast, table),
        syntax_tree::Factor::CondExpr(cond) => check_conditional_expression(cond, table),
        syntax_tree::Factor::Const(val) => Ok(check_const(val)),
        syntax_tree::Factor::FuncCall(func) => check_function_call(func, table),
        syntax_tree::Factor::HighPrecedence(expr) => type_check(expr, table),
        syntax_tree::Factor::Id(name) => check_id(name, table),
        syntax_tree::Factor::UnaryOp(unary) => check_unary_operator(unary, table),
    }
}

fn check_cast<'a>(
    cast: &'a syntax_tree::CastExpr,
    table: &'a LocalVariableTable,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    match cast {
        syntax_tree::CastExpr::Integer(expr) => {
            let kind = type_check(expr, table)?;
            if kind == syntax_tree::Kind::Real {
                Ok(syntax_tree::Kind::Int)
            } else {
                let err = SemanticError::CastError(CastError::ToInt(kind));
                Err(err)
            }
        }
        syntax_tree::CastExpr::Real(expr) => {
            let kind = type_check(expr, table)?;
            if kind == syntax_tree::Kind::Int {
                Ok(syntax_tree::Kind::Real)
            } else {
                let err = SemanticError::CastError(CastError::ToReal(kind));
                Err(err)
            }
        }
    }
}

fn check_conditional_expression<'a>(
    cond_expr: &'a syntax_tree::CondExpr,
    table: &'a LocalVariableTable,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    let cond_kind = type_check(&cond_expr.cond, table)?;
    if cond_kind == syntax_tree::Kind::Bool {
        let true_kind = type_check(&cond_expr.true_stat, table)?;
        let false_kind = type_check(&cond_expr.false_stat, table)?;
        if true_kind == false_kind {
            Ok(true_kind)
        } else {
            let err = MismatchedTypes::new(true_kind, false_kind);
            Err(SemanticError::MismatchedConditionalExpression(err))
        }
    } else {
        let err = NonBooleanCondition::CondStat(cond_kind);
        Err(SemanticError::NonBooleanCondition(err))
    }
}

fn check_const<'a>(value: &'a syntax_tree::Const) -> syntax_tree::Kind {
    match value {
        syntax_tree::Const::BoolConst(_) => syntax_tree::Kind::Bool,
        syntax_tree::Const::IntConst(_) => syntax_tree::Kind::Int,
        syntax_tree::Const::RealConst(_) => syntax_tree::Kind::Real,
        syntax_tree::Const::StrConst(_) => syntax_tree::Kind::Str,
    }
}

fn check_id<'a>(
    name: &'a str,
    table: &'a LocalVariableTable,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    match table.get_variable(name) {
        Ok(k) => Ok(k.clone()),
        Err(err) => Err(err),
    }
}

fn check_unary_operator<'a>(
    unary: &'a syntax_tree::UnaryOp,
    table: &'a LocalVariableTable,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    match unary {
        syntax_tree::UnaryOp::Minus(fact) => {
            let kind = check_factor(fact, table)?;
            match kind {
                syntax_tree::Kind::Int => Ok(syntax_tree::Kind::Int),
                syntax_tree::Kind::Real => Ok(syntax_tree::Kind::Real),
                other => Err(SemanticError::MismatchedUnary(MismatchedUnary::Numeric(
                    other,
                ))),
            }
        }
        syntax_tree::UnaryOp::Negate(fact) => {
            let kind = check_factor(fact, table)?;
            if kind == syntax_tree::Kind::Bool {
                Ok(syntax_tree::Kind::Bool)
            } else {
                Err(SemanticError::MismatchedUnary(MismatchedUnary::Logic(kind)))
            }
        }
    }
}

fn check_function_call<'a>(
    fcall: &'a syntax_tree::FuncCall,
    table: &'a LocalVariableTable,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    let func_proto = table.get_function(&fcall.id)?;
    if func_proto.params.len() == fcall.args.len() {
        for (formal, actual) in func_proto.params.iter().zip(fcall.args.iter()) {
            let actual_kind = type_check(actual, table)?;
            if actual_kind != formal.kind {
                let err = SemanticError::MismatchedArgumentType(MismatchedArgumentType::new(
                    &fcall.id,
                    formal.kind.clone(),
                    actual_kind,
                ));
                return Err(err);
            }
        }
        Ok(func_proto.kind.clone())
    } else {
        let err = SemanticError::ArgumentCountError(ArgumentCountError::new(
            &fcall.id,
            func_proto.params.len(),
            fcall.args.len(),
        ));
        Err(err)
    }
}

#[cfg(test)]
mod test {

    use super::super::name_table::{name_table_factory, VariableTable};
    use super::syntax_tree::*;
    use super::*;

    #[test]
    fn test_check_function_call() {
        let mut table = name_table_factory();
        let var_name = "value";
        table.insert_variable(var_name, &Kind::Int).unwrap();

        let func_name_a = "test";
        let func_decl_a = FuncDecl::new(func_name_a.to_owned(), vec![], Kind::Int, vec![], vec![]);

        let func_name_b = "do_stuff";
        let func_decl_b = FuncDecl::new(
            func_name_b.to_owned(),
            vec![ParamDecl::new("arg".to_owned(), Kind::Str)],
            Kind::Real,
            vec![],
            vec![],
        );

        let mut table = table.switch_to_function_table();
        table.insert_function(func_name_a, &func_decl_a).unwrap();
        table.insert_function(func_name_b, &func_decl_b).unwrap();

        let table_factory = table.switch_to_local_table();
        let table = table_factory.factory_local_table();

        let func_call = FuncCall::new(func_name_a.to_owned(), vec![]);

        let stat = check_function_call(&func_call, &table).unwrap();
        assert_eq!(stat, Kind::Int);

        let func_call = FuncCall::new(
            func_name_a.to_owned(),
            vec![Expr::Factor(Factor::Id(var_name.to_owned()))],
        );
        let stat = check_function_call(&func_call, &table);
        check_error_status(
            stat,
            SemanticError::ArgumentCountError(ArgumentCountError::new(func_name_a, 0, 1)),
        );

        let func_call = FuncCall::new(
            func_name_b.to_owned(),
            vec![Expr::Factor(Factor::Id(var_name.to_owned()))],
        );
        let stat = check_function_call(&func_call, &table);
        check_error_status(
            stat,
            SemanticError::MismatchedArgumentType(MismatchedArgumentType::new(
                func_name_b,
                Kind::Str,
                Kind::Int,
            )),
        );
    }

    #[test]
    fn test_check_cast() {
        let int_var_name = "int_var";
        let float_var_name = "float_var";

        let mut table = name_table_factory();
        table.insert_variable(int_var_name, &Kind::Int).unwrap();
        table.insert_variable(float_var_name, &Kind::Real).unwrap();

        let table_factory = table.switch_to_function_table().switch_to_local_table();
        let table = table_factory.factory_local_table();

        let correct_real_cast =
            CastExpr::Real(Box::new(Expr::Factor(Factor::Id(int_var_name.to_owned()))));
        let correct_int_cast = CastExpr::Integer(Box::new(Expr::Factor(Factor::Id(
            float_var_name.to_owned(),
        ))));

        assert_eq!(check_cast(&correct_real_cast, &table), Ok(Kind::Real));
        assert_eq!(check_cast(&correct_int_cast, &table), Ok(Kind::Int));

        let wrong_int_cast =
            CastExpr::Integer(Box::new(Expr::Factor(Factor::Id(int_var_name.to_owned()))));
        let wrong_real_cast = CastExpr::Real(Box::new(Expr::Factor(Factor::Id(
            float_var_name.to_owned(),
        ))));

        assert_eq!(
            check_cast(&wrong_int_cast, &table),
            Err(SemanticError::CastError(CastError::ToInt(Kind::Int)))
        );
        assert_eq!(
            check_cast(&wrong_real_cast, &table),
            Err(SemanticError::CastError(CastError::ToReal(Kind::Real)))
        );
    }

    #[test]
    fn test_unary_operator() {
        let table_factory = name_table_factory()
            .switch_to_function_table()
            .switch_to_local_table();

        let table = table_factory.factory_local_table();

        let correct_numeric_expr = Box::new(Expr::Node(
            Box::new(Expr::Factor(Factor::Const(Const::IntConst(4)))),
            Operator::Add,
            Box::new(Expr::Factor(Factor::Const(Const::IntConst(12)))),
        ));
        let correct_boolean_expr = Box::new(Expr::Node(
            Box::new(Expr::Factor(Factor::Const(Const::BoolConst(false)))),
            Operator::Or,
            Box::new(Expr::Factor(Factor::Const(Const::BoolConst(true)))),
        ));

        let correct_numeric_unary =
            UnaryOp::Minus(Box::new(Factor::HighPrecedence(correct_numeric_expr)));
        let correct_boolean_unary =
            UnaryOp::Negate(Box::new(Factor::HighPrecedence(correct_boolean_expr)));

        assert_eq!(
            check_unary_operator(&correct_numeric_unary, &table),
            Ok(Kind::Int)
        );
        assert_eq!(
            check_unary_operator(&correct_boolean_unary, &table),
            Ok(Kind::Bool)
        );

        let correct_numeric_expr = extract_content(correct_numeric_unary);
        let correct_boolean_expr = extract_content(correct_boolean_unary);

        let wrong_numertic_unary = UnaryOp::Minus(correct_boolean_expr);
        let wrong_boolean_unary = UnaryOp::Negate(correct_numeric_expr);

        assert_eq!(
            check_unary_operator(&wrong_numertic_unary, &table),
            Err(SemanticError::MismatchedUnary(MismatchedUnary::Numeric(
                Kind::Bool
            )))
        );

        assert_eq!(
            check_unary_operator(&wrong_boolean_unary, &table),
            Err(SemanticError::MismatchedUnary(MismatchedUnary::Logic(
                Kind::Int
            )))
        );
    }

    #[test]
    fn test_conditional_expression() {
        let mut table = name_table_factory();
        let int_var_name = "int_var";
        let real_var_name = "real_var";

        table.insert_variable(int_var_name, &Kind::Int).unwrap();
        table.insert_variable(real_var_name, &Kind::Real).unwrap();

        let mut table = table.switch_to_function_table();

        let str_func_name = "str_function";
        let str_func = FuncDecl::new(
            str_func_name.to_owned(),
            vec![ParamDecl::new("n".to_owned(), Kind::Int)],
            Kind::Str,
            vec![],
            vec![],
        );

        let void_func_name = "void_function";
        let void_func = FuncDecl::new(
            void_func_name.to_owned(),
            vec![],
            Kind::Void,
            vec![],
            vec![],
        );

        table.insert_function(str_func_name, &str_func).unwrap();
        table.insert_function(void_func_name, &void_func).unwrap();

        let correct_cond = CondExpr::new(
            Expr::Node(
                Box::new(Expr::Factor(Factor::Id(real_var_name.to_owned()))),
                Operator::Greater,
                Box::new(Expr::Factor(Factor::Const(Const::RealConst(4.5)))),
            ),
            Expr::Factor(Factor::FuncCall(FuncCall::new(
                str_func_name.to_owned(),
                vec![Expr::Node(
                    Box::new(Expr::Factor(Factor::Id(int_var_name.to_owned()))),
                    Operator::Mul,
                    Box::new(Expr::Factor(Factor::Const(Const::IntConst(21)))),
                )],
            ))),
            Expr::Factor(Factor::Const(Const::StrConst("test".to_owned()))),
        );

        let table_factory = table.switch_to_local_table();
        let table = table_factory.factory_local_table();
        assert_eq!(
            check_conditional_expression(&correct_cond, &table),
            Ok(Kind::Str)
        );

        let mismatched_cond = CondExpr::new(
            Expr::Node(
                Box::new(Expr::Factor(Factor::Id(real_var_name.to_owned()))),
                Operator::Greater,
                Box::new(Expr::Factor(Factor::Const(Const::RealConst(4.5)))),
            ),
            Expr::Factor(Factor::FuncCall(FuncCall::new(
                str_func_name.to_owned(),
                vec![Expr::Node(
                    Box::new(Expr::Factor(Factor::Id(int_var_name.to_owned()))),
                    Operator::Mul,
                    Box::new(Expr::Factor(Factor::Const(Const::IntConst(21)))),
                )],
            ))),
            Expr::Factor(Factor::Const(Const::IntConst(41))),
        );

        assert_eq!(
            check_conditional_expression(&mismatched_cond, &table),
            Err(SemanticError::MismatchedConditionalExpression(
                MismatchedTypes::new(Kind::Str, Kind::Int)
            ))
        );

        let non_bool_cond = CondExpr::new(
            Expr::Node(
                Box::new(Expr::Factor(Factor::Id(real_var_name.to_owned()))),
                Operator::Add,
                Box::new(Expr::Factor(Factor::Const(Const::RealConst(4.5)))),
            ),
            Expr::Factor(Factor::FuncCall(FuncCall::new(
                str_func_name.to_owned(),
                vec![Expr::Node(
                    Box::new(Expr::Factor(Factor::Id(int_var_name.to_owned()))),
                    Operator::Mul,
                    Box::new(Expr::Factor(Factor::Const(Const::IntConst(21)))),
                )],
            ))),
            Expr::Factor(Factor::Const(Const::StrConst("test".to_owned()))),
        );

        assert_eq!(
            check_conditional_expression(&non_bool_cond, &table),
            Err(SemanticError::NonBooleanCondition(
                NonBooleanCondition::CondStat(Kind::Real)
            ))
        )
    }

    #[test]
    fn test_coherent_operation() {
        for op in &[Operator::Add, Operator::Sub, Operator::Mul, Operator::Div] {
            for kind in &[Kind::Real, Kind::Int] {
                run_correct_coherent_test(kind, op, kind, kind);
            }
            for kind in &[Kind::Bool, Kind::Str] {
                run_inchoerent_operation(kind, op, kind)
            }
        }

        for op in &[
            Operator::Equal,
            Operator::NotEqual,
            Operator::Less,
            Operator::LessEqual,
            Operator::Greater,
            Operator::GreaterEqual,
        ] {
            for kind in &[Kind::Real, Kind::Int] {
                run_correct_coherent_test(kind, op, kind, &Kind::Bool);
            }

            run_inchoerent_operation(&Kind::Bool, op, &Kind::Bool);
        }

        for op in &[Operator::And, Operator::Or] {
            run_correct_coherent_test(&Kind::Bool, op, &Kind::Bool, &Kind::Bool);
            for kind in &[Kind::Int, Kind::Real, Kind::Str] {
                run_inchoerent_operation(kind, op, kind);
            }
        }

        // in this case the only error is mismatched types
        run_mismatched_types_test(&Kind::Real, &Operator::Sub, &Kind::Int);

        //here also the operation is not applicable, but the mismatch is more important
        run_mismatched_types_test(&Kind::Bool, &Operator::Less, &Kind::Int);

        run_mismatched_types_test(&Kind::Int, &Operator::Add, &Kind::Real);
    }

    fn check_error_status(res: Result<Kind, SemanticError>, expected: SemanticError) {
        match res {
            Ok(_) => panic!("{:?} is not an error"),
            Err(err) => assert_eq!(err, expected),
        }
    }

    fn run_correct_coherent_test(left: &Kind, op: &Operator, right: &Kind, expected: &Kind) {
        let stat = coherent_operation(left.clone(), op, right.clone());
        match stat {
            Ok(kind) => assert_eq!(&kind, expected),
            Err(err) => panic!(
                "This test[{:?}, {:?}, {:?}] generates an error: {:?}",
                left, op, right, err
            ),
        };
    }

    fn run_mismatched_types_test(left: &Kind, op: &Operator, right: &Kind) {
        let stat = coherent_operation(left.clone(), op, right.clone());
        match stat {
            Ok(_) => panic!("This test should fail: [{:?}, {:?}, {:?}]", left, op, right),
            Err(err) => match err {
                SemanticError::MismatchedOperationTypes(mismatch) => {
                    assert_eq!(&mismatch.left, left);
                    assert_eq!(&mismatch.right, right);
                },
                _ => panic!("This test[{:?}, {:?}, {:?}] generates an error: {:?}.\nShould Generate a MistmatchedOperationTypes", left, op, right, err)
            }
        }
    }

    fn run_inchoerent_operation(left: &Kind, op: &Operator, right: &Kind) {
        let stat = coherent_operation(left.clone(), op, right.clone());
        match stat {
            Ok(_) => panic!("This test should fail: [{:?}, {:?}, {:?}]", left, op, right),
            Err(err) => match err {
                SemanticError::IncoherentOperation(incoherent) => {
                    assert_eq!(&incoherent.var_kind, left);
                    assert_eq!(&incoherent.operator, op);
                },
                _ => panic!("This test[{:?}, {:?}, {:?}] generates an error: {:?}.\nShould Generate a MistmatchedOperationTypes", left, op, right, err)
            }
        }
    }

    fn extract_content(unary: UnaryOp) -> Box<Factor> {
        match unary {
            UnaryOp::Minus(out) => out,
            UnaryOp::Negate(out) => out,
        }
    }
}
