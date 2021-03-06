use super::name_table::LocalVariableTable;
use super::semantic_error::{
    ArgumentCountError, CastError, IncoherentOperation, MismatchedArgumentType, MismatchedTypes,
    MismatchedUnary, NonBooleanCondition, SemanticError,
};
use simpla_parser::syntax_tree;

pub fn function_call_check<'a>(
    func_call: &'a syntax_tree::FuncCall,
    table: &'a LocalVariableTable<'a>,
    loc: &'a syntax_tree::Location,
) -> Result<(), SemanticError<'a>> {
    check_function_call(func_call, table, loc)?;
    Ok(())
}

pub fn type_check<'a>(
    expr: &'a syntax_tree::Expr,
    table: &'a LocalVariableTable,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    let kind = match &expr.expr {
        syntax_tree::ExprTree::Node(left, op, right) => {
            let left_type = type_check(left, table)?;
            let right_type = type_check(right, table)?;
            let output = coherent_operation(left_type, op, right_type, &expr.loc)?;
            Ok(output)
        }
        syntax_tree::ExprTree::Factor(fact) => check_factor(fact, table, &expr.loc),
    }?;
    *expr.kind.borrow_mut() = Some(kind.clone());
    Ok(kind)
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
    loc: &'a syntax_tree::Location,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    if left == right {
        let op_kind = OperatorKind::from_operator(op);
        match op_kind {
            OperatorKind::Logic => match left {
                syntax_tree::Kind::Bool => Ok(left),
                _ => {
                    let err = IncoherentOperation::new(left, op.clone(), loc);
                    Err(SemanticError::IncoherentOperation(err))
                }
            },
            OperatorKind::Numeric => match left {
                syntax_tree::Kind::Int | syntax_tree::Kind::Real => Ok(left),
                _ => {
                    let err = IncoherentOperation::new(left, op.clone(), loc);
                    Err(SemanticError::IncoherentOperation(err))
                }
            },
            OperatorKind::Relational => match left {
                syntax_tree::Kind::Int
                | syntax_tree::Kind::Real
                | syntax_tree::Kind::Bool
                | syntax_tree::Kind::Str => Ok(syntax_tree::Kind::Bool),
                _ => {
                    let err = IncoherentOperation::new(left, op.clone(), loc);
                    Err(SemanticError::IncoherentOperation(err))
                }
            },
        }
    } else {
        let mismatch = MismatchedTypes::new(left, right, loc);
        Err(SemanticError::MismatchedOperationTypes(mismatch))
    }
}

fn check_factor<'a>(
    fact: &'a syntax_tree::Factor,
    table: &'a LocalVariableTable,
    loc: &'a syntax_tree::Location,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    let kind = match &fact.fact {
        syntax_tree::FactorValue::CastExpr(cast) => check_cast(cast, table, loc),
        syntax_tree::FactorValue::CondExpr(cond) => check_conditional_expression(cond, table, loc),
        syntax_tree::FactorValue::Const(val) => Ok(check_const(val)),
        syntax_tree::FactorValue::FuncCall(func) => check_function_call(func, table, loc),
        syntax_tree::FactorValue::HighPrecedence(expr) => type_check(expr, table),
        syntax_tree::FactorValue::Id(name) => check_id(name, table),
        syntax_tree::FactorValue::UnaryOp(unary) => check_unary_operator(unary, table, loc),
    }?;

    *fact.kind.borrow_mut() = Some(kind.clone());
    Ok(kind)
}

fn check_cast<'a>(
    cast: &'a syntax_tree::CastExpr,
    table: &'a LocalVariableTable,
    loc: &'a syntax_tree::Location,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    match cast {
        syntax_tree::CastExpr::Integer(expr) => {
            let kind = type_check(expr, table)?;
            if kind == syntax_tree::Kind::Real {
                Ok(syntax_tree::Kind::Int)
            } else {
                let err = SemanticError::CastError(CastError::new_to_int(loc, kind));
                Err(err)
            }
        }
        syntax_tree::CastExpr::Real(expr) => {
            let kind = type_check(expr, table)?;
            if kind == syntax_tree::Kind::Int {
                Ok(syntax_tree::Kind::Real)
            } else {
                let err = SemanticError::CastError(CastError::new_to_real(loc, kind));
                Err(err)
            }
        }
    }
}

fn check_conditional_expression<'a>(
    cond_expr: &'a syntax_tree::CondExpr,
    table: &'a LocalVariableTable,
    loc: &'a syntax_tree::Location,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    let cond_kind = type_check(&cond_expr.cond, table)?;
    if cond_kind == syntax_tree::Kind::Bool {
        let true_kind = type_check(&cond_expr.true_stat, table)?;
        let false_kind = type_check(&cond_expr.false_stat, table)?;
        if true_kind == false_kind {
            Ok(true_kind)
        } else {
            let err = MismatchedTypes::new(true_kind, false_kind, loc);
            Err(SemanticError::MismatchedConditionalExpression(err))
        }
    } else {
        let err = NonBooleanCondition::new_cond_stat(loc, cond_kind);
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
    loc: &'a syntax_tree::Location,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    match unary {
        syntax_tree::UnaryOp::Minus(fact) => {
            let kind = check_factor(fact, table, loc)?;
            match kind {
                syntax_tree::Kind::Int => Ok(syntax_tree::Kind::Int),
                syntax_tree::Kind::Real => Ok(syntax_tree::Kind::Real),
                other => Err(SemanticError::MismatchedUnary(
                    MismatchedUnary::new_numeric(loc, other),
                )),
            }
        }
        syntax_tree::UnaryOp::Negate(fact) => {
            let kind = check_factor(fact, table, loc)?;
            if kind == syntax_tree::Kind::Bool {
                Ok(syntax_tree::Kind::Bool)
            } else {
                Err(SemanticError::MismatchedUnary(MismatchedUnary::new_logic(
                    loc, kind,
                )))
            }
        }
    }
}

fn check_function_call<'a>(
    fcall: &'a syntax_tree::FuncCall,
    table: &'a LocalVariableTable,
    loc: &'a syntax_tree::Location,
) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    let func_proto = table.get_function(&fcall.id)?;
    if func_proto.params.len() == fcall.args.len() {
        for (i, (formal, actual)) in func_proto.params.iter().zip(fcall.args.iter()).enumerate() {
            let actual_kind = type_check(actual, table)?;
            if actual_kind != formal.kind {
                let err = SemanticError::MismatchedArgumentType(MismatchedArgumentType::new(
                    func_proto,
                    formal.kind.clone(),
                    actual_kind,
                    i,
                    loc,
                ));
                return Err(err);
            }
        }
        Ok(func_proto.kind.clone())
    } else {
        let err = SemanticError::ArgumentCountError(ArgumentCountError::new(&func_proto, &fcall));
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
        let fake_location = syntax_tree::Location::new(0, 0);
        let mut table = name_table_factory();
        let var_name = "value";
        let loc = Location::new(123, 456);
        table.insert_variable(var_name, &Kind::Int, &loc).unwrap();

        let func_name_a = "test";
        let func_decl_a = FuncDecl::new(
            func_name_a.to_owned(),
            vec![],
            Kind::Int,
            vec![],
            vec![],
            0,
            0,
        );

        let func_name_b = "do_stuff";
        let func_decl_b = FuncDecl::new(
            func_name_b.to_owned(),
            vec![ParamDecl::new("arg".to_owned(), Kind::Str)],
            Kind::Real,
            vec![],
            vec![],
            0,
            0,
        );

        let mut table = table.switch_to_function_table();
        table.insert_function(func_name_a, &func_decl_a).unwrap();
        table.insert_function(func_name_b, &func_decl_b).unwrap();

        let table_factory = table.switch_to_local_table();
        let table = table_factory.factory_local_table();

        let func_call = FuncCall::new(func_name_a.to_owned(), vec![]);

        let stat = check_function_call(&func_call, &table, &Location::new(0, 0)).unwrap();
        assert_eq!(stat, Kind::Int);

        let func_call = FuncCall::new(
            func_name_a.to_owned(),
            vec![Expr::new(
                ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                    var_name.to_owned(),
                ))),
                0,
                0,
            )],
        );
        let stat = check_function_call(&func_call, &table, &fake_location);
        check_error_status(
            stat,
            SemanticError::ArgumentCountError(ArgumentCountError::new(&func_decl_a, &func_call)),
        );

        let func_call = FuncCall::new(
            func_name_b.to_owned(),
            vec![Expr::new(
                ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                    var_name.to_owned(),
                ))),
                0,
                0,
            )],
        );
        let stat = check_function_call(&func_call, &table, &fake_location);
        check_error_status(
            stat,
            SemanticError::MismatchedArgumentType(MismatchedArgumentType::new(
                &func_decl_b,
                Kind::Str,
                Kind::Int,
                0,
                &fake_location,
            )),
        );
    }

    #[test]
    fn test_check_cast() {
        let int_var_name = "int_var";
        let float_var_name = "float_var";
        let loc_int = Location::new(123, 456);
        let loc_real = Location::new(234, 567);
        let mut table = name_table_factory();
        table
            .insert_variable(int_var_name, &Kind::Int, &loc_int)
            .unwrap();
        table
            .insert_variable(float_var_name, &Kind::Real, &loc_real)
            .unwrap();

        let table_factory = table.switch_to_function_table().switch_to_local_table();
        let table = table_factory.factory_local_table();

        let correct_real_cast = CastExpr::Real(Box::new(Expr::new(
            ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                int_var_name.to_owned(),
            ))),
            0,
            0,
        )));
        let correct_int_cast = CastExpr::Integer(Box::new(Expr::new(
            ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                float_var_name.to_owned(),
            ))),
            0,
            0,
        )));

        assert_eq!(
            check_cast(&correct_real_cast, &table, &Location::new(0, 0)),
            Ok(Kind::Real)
        );
        assert_eq!(
            check_cast(&correct_int_cast, &table, &Location::new(0, 0)),
            Ok(Kind::Int)
        );

        let wrong_int_cast = CastExpr::Integer(Box::new(Expr::new(
            ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                int_var_name.to_owned(),
            ))),
            0,
            0,
        )));
        let wrong_real_cast = CastExpr::Real(Box::new(Expr::new(
            ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                float_var_name.to_owned(),
            ))),
            0,
            0,
        )));

        assert_eq!(
            check_cast(&wrong_int_cast, &table, &loc_int),
            Err(SemanticError::CastError(CastError::new_to_int(
                &loc_int,
                Kind::Int
            )))
        );
        assert_eq!(
            check_cast(&wrong_real_cast, &table, &loc_real),
            Err(SemanticError::CastError(CastError::new_to_real(
                &loc_real,
                Kind::Real
            )))
        );
    }

    #[test]
    fn test_unary_operator() {
        let table_factory = name_table_factory()
            .switch_to_function_table()
            .switch_to_local_table();

        let table = table_factory.factory_local_table();

        let correct_numeric_expr = Box::new(Expr::new(
            ExprTree::Node(
                Box::new(Expr::new(
                    ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                        Const::IntConst(4),
                    ))),
                    0,
                    0,
                )),
                Operator::Add,
                Box::new(Expr::new(
                    ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                        Const::IntConst(12),
                    ))),
                    0,
                    0,
                )),
            ),
            0,
            10,
        ));
        let correct_boolean_expr = Box::new(Expr::new(
            ExprTree::Node(
                Box::new(Expr::new(
                    ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                        Const::BoolConst(false),
                    ))),
                    0,
                    0,
                )),
                Operator::Or,
                Box::new(Expr::new(
                    ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                        Const::BoolConst(true),
                    ))),
                    0,
                    0,
                )),
            ),
            0,
            0,
        ));

        let correct_numeric_unary = UnaryOp::Minus(Box::new(syntax_tree::Factor::new(
            FactorValue::HighPrecedence(correct_numeric_expr),
        )));
        let correct_boolean_unary = UnaryOp::Negate(Box::new(syntax_tree::Factor::new(
            FactorValue::HighPrecedence(correct_boolean_expr),
        )));

        assert_eq!(
            check_unary_operator(&correct_numeric_unary, &table, &Location::new(0, 0)),
            Ok(Kind::Int)
        );
        assert_eq!(
            check_unary_operator(&correct_boolean_unary, &table, &Location::new(0, 0)),
            Ok(Kind::Bool)
        );

        let correct_numeric_expr = extract_content(correct_numeric_unary);
        let correct_boolean_expr = extract_content(correct_boolean_unary);

        let wrong_numertic_unary = UnaryOp::Minus(correct_boolean_expr);
        let wrong_boolean_unary = UnaryOp::Negate(correct_numeric_expr);

        assert_eq!(
            check_unary_operator(&wrong_numertic_unary, &table, &Location::new(0, 0)),
            Err(SemanticError::MismatchedUnary(
                MismatchedUnary::new_numeric(&Location::new(0, 0), syntax_tree::Kind::Bool)
            ))
        );

        assert_eq!(
            check_unary_operator(&wrong_boolean_unary, &table, &Location::new(0, 0)),
            Err(SemanticError::MismatchedUnary(MismatchedUnary::new_logic(
                &Location::new(0, 0),
                Kind::Int
            )))
        );
    }

    #[test]
    fn test_conditional_expression() {
        let mut table = name_table_factory();
        let int_var_name = "int_var";
        let real_var_name = "real_var";

        let loc_int = Location::new(123, 456);
        let loc_real = Location::new(234, 567);

        table
            .insert_variable(int_var_name, &Kind::Int, &loc_int)
            .unwrap();
        table
            .insert_variable(real_var_name, &Kind::Real, &loc_real)
            .unwrap();

        let mut table = table.switch_to_function_table();

        let str_func_name = "str_function";
        let str_func = FuncDecl::new(
            str_func_name.to_owned(),
            vec![ParamDecl::new("n".to_owned(), Kind::Int)],
            Kind::Str,
            vec![],
            vec![],
            0,
            0,
        );

        let void_func_name = "void_function";
        let void_func = FuncDecl::new(
            void_func_name.to_owned(),
            vec![],
            Kind::Void,
            vec![],
            vec![],
            0,
            0,
        );

        table.insert_function(str_func_name, &str_func).unwrap();
        table.insert_function(void_func_name, &void_func).unwrap();

        let correct_cond = CondExpr::new(
            Expr::new(
                ExprTree::Node(
                    Box::new(Expr::new(
                        ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                            real_var_name.to_owned(),
                        ))),
                        0,
                        0,
                    )),
                    Operator::Greater,
                    Box::new(Expr::new(
                        ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                            Const::RealConst(4.5),
                        ))),
                        0,
                        0,
                    )),
                ),
                10,
                15,
            ),
            Expr::new(
                ExprTree::Factor(syntax_tree::Factor::new(FactorValue::FuncCall(
                    FuncCall::new(
                        str_func_name.to_owned(),
                        vec![Expr::new(
                            ExprTree::Node(
                                Box::new(Expr::new(
                                    ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                                        int_var_name.to_owned(),
                                    ))),
                                    0,
                                    0,
                                )),
                                Operator::Mul,
                                Box::new(Expr::new(
                                    ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                                        Const::IntConst(21),
                                    ))),
                                    0,
                                    0,
                                )),
                            ),
                            56,
                            156,
                        )],
                    ),
                ))),
                0,
                0,
            ),
            Expr::new(
                ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                    Const::StrConst("test".to_owned()),
                ))),
                0,
                0,
            ),
        );

        let table_factory = table.switch_to_local_table();
        let table = table_factory.factory_local_table();
        assert_eq!(
            check_conditional_expression(&correct_cond, &table, &Location::new(0, 0)),
            Ok(Kind::Str)
        );

        let mismatched_cond = CondExpr::new(
            Expr::new(
                ExprTree::Node(
                    Box::new(Expr::new(
                        ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                            real_var_name.to_owned(),
                        ))),
                        0,
                        0,
                    )),
                    Operator::Greater,
                    Box::new(Expr::new(
                        ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                            Const::RealConst(4.5),
                        ))),
                        0,
                        0,
                    )),
                ),
                56,
                125,
            ),
            Expr::new(
                ExprTree::Factor(syntax_tree::Factor::new(FactorValue::FuncCall(
                    FuncCall::new(
                        str_func_name.to_owned(),
                        vec![Expr::new(
                            ExprTree::Node(
                                Box::new(Expr::new(
                                    ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                                        int_var_name.to_owned(),
                                    ))),
                                    0,
                                    0,
                                )),
                                Operator::Mul,
                                Box::new(Expr::new(
                                    ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                                        Const::IntConst(21),
                                    ))),
                                    0,
                                    0,
                                )),
                            ),
                            0,
                            0,
                        )],
                    ),
                ))),
                156,
                234,
            ),
            Expr::new(
                ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                    Const::IntConst(41),
                ))),
                0,
                0,
            ),
        );

        assert_eq!(
            check_conditional_expression(&mismatched_cond, &table, &Location::new(0, 0)),
            Err(SemanticError::MismatchedConditionalExpression(
                MismatchedTypes::new(Kind::Str, Kind::Int, &Location::new(0, 0))
            ))
        );

        let non_bool_cond = CondExpr::new(
            Expr::new(
                ExprTree::Node(
                    Box::new(Expr::new(
                        ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                            real_var_name.to_owned(),
                        ))),
                        0,
                        0,
                    )),
                    Operator::Add,
                    Box::new(Expr::new(
                        ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                            Const::RealConst(4.5),
                        ))),
                        0,
                        0,
                    )),
                ),
                10,
                24,
            ),
            Expr::new(
                ExprTree::Factor(syntax_tree::Factor::new(FactorValue::FuncCall(
                    FuncCall::new(
                        str_func_name.to_owned(),
                        vec![Expr::new(
                            ExprTree::Node(
                                Box::new(Expr::new(
                                    ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Id(
                                        int_var_name.to_owned(),
                                    ))),
                                    0,
                                    0,
                                )),
                                Operator::Mul,
                                Box::new(Expr::new(
                                    ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                                        Const::IntConst(21),
                                    ))),
                                    0,
                                    0,
                                )),
                            ),
                            56,
                            100,
                        )],
                    ),
                ))),
                56,
                100,
            ),
            Expr::new(
                ExprTree::Factor(syntax_tree::Factor::new(FactorValue::Const(
                    Const::StrConst("test".to_owned()),
                ))),
                0,
                0,
            ),
        );

        assert_eq!(
            check_conditional_expression(&non_bool_cond, &table, &Location::new(0, 0)),
            Err(SemanticError::NonBooleanCondition(
                NonBooleanCondition::new_cond_stat(&syntax_tree::Location::new(0, 0), Kind::Real)
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
            for kind in &[Kind::Real, Kind::Int, Kind::Bool, Kind::Str] {
                run_correct_coherent_test(kind, op, kind, &Kind::Bool);
            }
        }

        for op in &[Operator::And, Operator::Or] {
            run_correct_coherent_test(&Kind::Bool, op, &Kind::Bool, &Kind::Bool);
            for kind in &[Kind::Int, Kind::Real, Kind::Str] {
                run_inchoerent_operation(kind, op, kind);
            }
        }

        // in this case the only error is mismatched types
        run_mismatched_types_test(&Kind::Real, &Operator::Sub, &Kind::Int);

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
        let loc = Location::new(10, 15);
        let stat = coherent_operation(left.clone(), op, right.clone(), &loc);
        match stat {
            Ok(kind) => assert_eq!(&kind, expected),
            Err(err) => panic!(
                "This test[{:?}, {:?}, {:?}] generates an error: {:?}",
                left, op, right, err
            ),
        };
    }

    fn run_mismatched_types_test(left: &Kind, op: &Operator, right: &Kind) {
        let loc = Location::new(10, 15);
        let stat = coherent_operation(left.clone(), op, right.clone(), &loc);
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
        let loc = Location::new(10, 15);
        let stat = coherent_operation(left.clone(), op, right.clone(), &loc);
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
