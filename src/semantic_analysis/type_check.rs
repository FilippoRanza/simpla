use super::name_table::LocalVariableTable;
use super::semantic_error::{
    CastError, IncoherentOperation, MismatchedTypes, MismatchedUnary, NonBooleanCondition,
    SemanticError,
};
use simpla_parser::syntax_tree;

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
    Logic,
}

impl OperatorKind {
    fn from_operator(op: &syntax_tree::Operator) -> Self {
        match op {
            syntax_tree::Operator::And | syntax_tree::Operator::Or => Self::Logic,
            _ => Self::Numeric,
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
                let err = SemanticError::MismatchedArgumentType {
                    func_name: &fcall.id,
                    correct: formal.kind.clone(),
                    given: actual_kind,
                };
                return Err(err);
            }
        }
        Ok(func_proto.kind.clone())
    } else {
        let err = SemanticError::ArgumentCountError {
            func_name: &fcall.id,
            correct: func_proto.params.len(),
            given: fcall.args.len(),
        };
        Err(err)
    }
}

#[cfg(test)]
mod test {

    use super::super::name_table::name_table_factory;
    use super::syntax_tree::{Kind, Operator};
    use super::*;

    #[test]
    fn test_check_cast() {}

    #[test]
    fn test_coherent_operation() {
        for op in &[
            Operator::Add,
            Operator::Sub,
            Operator::Mul,
            Operator::Div,
            Operator::Equal,
            Operator::NotEqual,
            Operator::Less,
            Operator::LessEqual,
            Operator::Greater,
            Operator::GreaterEqual,
        ] {
            for kind in &[Kind::Real, Kind::Int] {
                run_correct_coherent_test(kind, op, kind);
            }
            for kind in &[Kind::Bool, Kind::Str] {
                run_inchoerent_operation(kind, op, kind)
            }
        }

        for op in &[Operator::And, Operator::Or] {
            run_correct_coherent_test(&Kind::Bool, op, &Kind::Bool);
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

    fn run_correct_coherent_test(left: &Kind, op: &Operator, right: &Kind) {
        let stat = coherent_operation(left.clone(), op, right.clone());
        match stat {
            Ok(_) => {}
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
}
