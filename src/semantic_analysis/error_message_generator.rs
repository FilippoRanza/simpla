use super::extract_wrong_code::format_wrong_code;
use super::semantic_error;
use simpla_parser::syntax_tree;

impl<'a> semantic_error::SemanticError<'a> {
    pub fn format_error(&self, code: &str) -> String {
        let msg = match self {
            Self::NameRidefinition(err) => format!("name error: {}", err.format_error(code)),
            Self::VoidVariableDeclaration(err) => {
                format!("void declaration error: {}", err.format_error(code))
            }
            Self::MismatchedOperationTypes(err) => {
                format!("mismatched operation error: {}", err.format_error(code))
            }
            Self::IncoherentOperation(err) => {
                format!("incoherent operation error: {}", err.format_error(code))
            }
            Self::CastError(err) => format!("cast error: {}", err.format_error(code)),
            Self::NonBooleanCondition(err) => {
                format!("condition error: {}", err.format_error(code))
            }
            Self::MismatchedConditionalExpression(err) => {
                format!("conditional expression error: {}", err.format_error(code))
            }
            Self::UnknownFunction(err) => format!("unknown function error: {}", err),
            Self::UnknownVariable(err) => format!("unknonw variable error: {}", err),
            Self::MismatchedUnary(err) => format!("negation error: {}", err.format_error(code)),
            Self::ArgumentCountError(err) => {
                format!("argument count error: {}", err.format_error(code))
            }
            Self::MismatchedArgumentType(err) => {
                format!("argument type error: {}", err.format_error(code))
            }
            Self::MismatchedAssignment(err) => {
                format!("assignment error: {}", err.format_error(code))
            }
            Self::BreakOutsideLoop => format!("break error: break outside loop"),
            Self::ForLoopError(err) => format!("for loop error: {}", err.format_error(code)),
            Self::ReturnError(err) => format!("return error: {}", err.format_error(code)),
        };
        format!("{}", msg)
    }
}
impl semantic_error::NameRidefinition {
    fn format_error(&self, code: &str) -> String {
        format!(
            "{} defined multiple times, originally: {}, redefined: {}",
            self.name,
            self.original.format_error(code),
            self.new.format_error(code)
        )
    }
}
impl semantic_error::Ridefinition {
    fn format_error(&self, code: &str) -> String {
        match self {
            Self::Function(loc) => format!("function here:\n{}", format_wrong_code(code, loc)),
            Self::Variable(loc) => format!("variable here:\n{}", format_wrong_code(code, loc)),
        }
    }
}
impl<'a> semantic_error::VoidVariableDeclaration<'a> {
    fn format_error(&self, code: &str) -> String {
        let tmp = self.names.id_list.join(", ");
        format!(
            "Variables: [{}] defined as type void: only function can have type void:\n{}",
            tmp,
            format_wrong_code(code, &self.names.loc)
        )
    }
}
impl<'a> semantic_error::MismatchedTypes<'a> {
    fn format_error(&self, code: &str) -> String {
        format!(
            "left type: {} right type: {}:\n{}",
            kind_to_string(&self.left),
            kind_to_string(&self.right),
            format_wrong_code(code, &self.loc)
        )
    }
}
impl<'a> semantic_error::IncoherentOperation<'a> {
    fn format_error(&self, code: &str) -> String {
        format!(
            "cannot apply operator {} to type {}\n{}",
            operator_to_string(&self.operator),
            kind_to_string(&self.var_kind),
            format_wrong_code(code, &self.loc)
        )
    }
}
impl<'a> semantic_error::CastError<'a> {
    fn format_error(&self, code: &str) -> String {
        let token = format_wrong_code(code, &self.loc);
        match &self.error {
            semantic_error::CastErrorType::ToInt(k) => {
                format!("cannot cast {} into integer:\n{}", kind_to_string(k), token)
            }
            semantic_error::CastErrorType::ToReal(k) => {
                format!("cannot cast {} into real:\n{}", kind_to_string(k), token)
            }
        }
    }
}
impl<'a> semantic_error::NonBooleanCondition<'a> {
    fn format_error(&self, code: &str) -> String {
        fn fmt_err(
            stat: &str,
            kind: &syntax_tree::Kind,
            code: &str,
            loc: &syntax_tree::Location,
        ) -> String {
            format!(
                "{} statement requires a boolean expression as condition, found: {}\n{}",
                stat,
                kind_to_string(kind),
                format_wrong_code(code, loc)
            )
        }
        match &self.error {
            semantic_error::NonBooleanConditionType::IfStat(k) => fmt_err("if", k, code, self.loc),
            semantic_error::NonBooleanConditionType::WhileStat(k) => {
                fmt_err("while", k, code, self.loc)
            }
            semantic_error::NonBooleanConditionType::CondStat(k) => {
                fmt_err("conditional", k, code, self.loc)
            }
        }
    }
}
impl<'a> semantic_error::MismatchedUnary<'a> {
    fn format_error(&self, code: &str) -> String {
        fn fmt_err(unary: &str, kind: &syntax_tree::Kind, err: String) -> String {
            format!(
                "{} cannot be applied to type: {}\n{}",
                unary,
                kind_to_string(kind),
                err
            )
        }
        let token = format_wrong_code(code, self.loc);
        match &self.error {
            semantic_error::MismatchedUnaryType::Logic(k) => fmt_err("logic negation", k, token),
            semantic_error::MismatchedUnaryType::Numeric(k) => {
                fmt_err("arithmetic negation", k, token)
            }
        }
    }
}
impl<'a> semantic_error::MismatchedAssignment<'a> {
    fn format_error(&self, code: &str) -> String {
        format!(
            "expected {}, found {} in variable {} assignment:\n{}",
            kind_to_string(&self.correct),
            kind_to_string(&self.given),
            self.name,
            format_wrong_code(code, self.loc)
        )
    }
}
impl<'a> semantic_error::ForLoopError<'a> {
    fn format_error(&self, code: &str) -> String {
        let descr = match &self.error {
            semantic_error::ForLoopErrorType::CountVariableAssignment(name) => {
                format!("count variable {} is modified into loop body", name)
            }
            semantic_error::ForLoopErrorType::NonIntegerCount(k) => format!(
                "count variable is declared as {}, expected integer",
                kind_to_string(k)
            ),
            semantic_error::ForLoopErrorType::NonIntegerStart(k) => format!(
                "for loop start expression of type {}, expected integer",
                kind_to_string(k)
            ),
            semantic_error::ForLoopErrorType::NonIntegerEnd(k) => format!(
                "for loop end expression of type {}, expected integer",
                kind_to_string(k)
            ),
        };
        let token = format_wrong_code(code, self.loc);
        format!("{}\n{}", descr, token)
    }
}
impl<'a> semantic_error::ReturnError<'a> {
    fn format_error(&self, code: &str) -> String {
        let token = format_wrong_code(code, self.loc);
        match &self.error {
            semantic_error::ReturnErrorType::ReturnOutsideFunction => format!(
                "return statement is not allowd in main body, only in function declaration:\n{}",
                token
            ),
            semantic_error::ReturnErrorType::MismatchedReturnType(correct, given) => format!(
                "return statement type: {}, but {} was expected:\n{}",
                kind_to_string(&correct),
                kind_to_string(&given),
                token
            ),
        }
    }
}
impl<'a> semantic_error::ArgumentCountError<'a> {
    fn format_error(&self, code: &str) -> String {
        format!(
            "function: {} expected {} args, but {} are used in function call:\nFunction '{1}' definition:\n{}\nFunction '{1}' call:\n{}",
            self.func_decl.id,
            self.func_decl.params.len(),
            self.func_call.args.len(),
            format_wrong_code(code, &self.func_decl.loc),
            format_wrong_code(code, &self.func_decl.loc)
        )
    }
}
impl<'a> semantic_error::MismatchedArgumentType<'a> {
    fn format_error(&self, code: &str) -> String {
        format!(
            "calling function {} argument expected type: {}, found {} in position {}:\nFunction declaration: {}\nFunction call: {}",
            self.func.id,
            kind_to_string(&self.correct),
            kind_to_string(&self.given),
            (self.index + 1),
            format_wrong_code(code, &self.func.loc),
            format_wrong_code(code, &self.loc)
        )
    }
}

fn kind_to_string(k: &syntax_tree::Kind) -> String {
    let output = match k {
        syntax_tree::Kind::Bool => "bool",
        syntax_tree::Kind::Int => "integer",
        syntax_tree::Kind::Real => "real",
        syntax_tree::Kind::Str => "string",
        syntax_tree::Kind::Void => "void",
    };
    output.to_owned()
}

fn operator_to_string(o: &syntax_tree::Operator) -> String {
    let output = match o {
        syntax_tree::Operator::Equal => "Equal",
        syntax_tree::Operator::NotEqual => "NotEqual",
        syntax_tree::Operator::Greater => "Greater",
        syntax_tree::Operator::GreaterEqual => "GreaterEqual",
        syntax_tree::Operator::Less => "Less",
        syntax_tree::Operator::LessEqual => "LessEqual",
        syntax_tree::Operator::Add => "Add",
        syntax_tree::Operator::Sub => "Sub",
        syntax_tree::Operator::Mul => "Mul",
        syntax_tree::Operator::Div => "Div",
        syntax_tree::Operator::And => "And",
        syntax_tree::Operator::Or => "Or",
    };
    output.to_owned()
}
