use super::semantic_error;
use simpla_parser::syntax_tree;
use std::convert;
use std::fmt;

impl<'a> convert::From<semantic_error::SemanticError<'a>> for String {
    fn from(err: semantic_error::SemanticError) -> String {
        format!("{}", err)
    }
}

impl<'a> fmt::Display for semantic_error::SemanticError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Self::NameRidefinition(err) => format!("name error: {}", err),
            Self::VoidVariableDeclaration(err) => format!("void declaration error: {}", err),
            Self::MismatchedOperationTypes(err) => {
                format!("mismatched operation error: {}", err)
            }
            Self::IncoherentOperation(err) => format!("incoherent operation error: {}", err),
            Self::CastError(err) => format!("cast error: {}", err),
            Self::NonBooleanCondition(err) => format!("condition error: {}", err),
            Self::MismatchedConditionalExpression(err) => {
                format!("conditional expression error: {}", err)
            }
            Self::UnknownFunction(err) => format!("unknown function error: {}", err),
            Self::UnknownVariable(err) => format!("unknonw variable error: {}", err),
            Self::MismatchedUnary(err) => format!("negation error: {}", err),
            Self::ArgumentCountError(err) => format!("argument count error: {}", err),
            Self::MismatchedArgumentType(err) => format!("argument type error: {}", err),
            Self::MismatchedAssignment(err) => format!("assignment error: {}", err),
            Self::BreakOutsideLoop => format!("break error: break outside loop"),
            Self::ForLoopError(err) => format!("for loop error: {}", err),
            Self::ReturnError(err) => format!("return error: {}", err),
        };
        write!(f, "{}", msg)
    }
}
impl fmt::Display for semantic_error::NameRidefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} defined multiple times, originally: {}, redefined: {}",
            self.name, self.original, self.new
        )
    }
}
impl fmt::Display for semantic_error::Ridefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Variable => write!(f, "variable"),
        }
    }
}
impl<'a> fmt::Display for semantic_error::VoidVariableDeclaration<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp = self.names.id_list.join(", ");
        write!(
            f,
            "Variables: [{}] defined as type void: only function can have type void",
            tmp
        )
    }
}
impl fmt::Display for semantic_error::MismatchedTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "left type: {} right type: {}",
            kind_to_string(&self.left),
            kind_to_string(&self.right)
        )
    }
}
impl fmt::Display for semantic_error::IncoherentOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "cannot apply operator {} to type {}",
            operator_to_string(&self.operator),
            kind_to_string(&self.var_kind)
        )
    }
}
impl fmt::Display for semantic_error::CastError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ToInt(k) => write!(f, "cannot cast {} into integer", kind_to_string(k)),
            Self::ToReal(k) => write!(f, "cannot cast {} into real", kind_to_string(k)),
        }
    }
}
impl fmt::Display for semantic_error::NonBooleanCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt(f: &mut fmt::Formatter, stat: &str, kind: &syntax_tree::Kind) -> fmt::Result {
            write!(
                f,
                "{} statement requires a boolean expression as condition, found: {}",
                stat,
                kind_to_string(kind)
            )
        }
        match self {
            Self::IfStat(k) => fmt(f, "if", k),
            Self::WhileStat(k) => fmt(f, "while", k),
            Self::CondStat(k) => fmt(f, "conditional", k),
        }
    }
}
impl fmt::Display for semantic_error::MismatchedUnary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt(f: &mut fmt::Formatter, unary: &str, kind: &syntax_tree::Kind) -> fmt::Result {
            write!(
                f,
                "{} cannot be applied to type: {}",
                unary,
                kind_to_string(kind)
            )
        }
        match self {
            Self::Logic(k) => fmt(f, "logic negation", k),
            Self::Numeric(k) => fmt(f, "arithmetic negation", k),
        }
    }
}
impl<'a> fmt::Display for semantic_error::MismatchedAssignment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "expected {}, found {} in variable {} assignment",
            kind_to_string(&self.correct),
            kind_to_string(&self.given),
            self.name
        )
    }
}
impl<'a> fmt::Display for semantic_error::ForLoopError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CountVariableAssignment(name) => {
                write!(f, "count variable {} is modified into loop body", name)
            }
            Self::NonIntegerCount(k) => write!(
                f,
                "count variable is declared as {}, expected integer",
                kind_to_string(k)
            ),
            Self::NonIntegerStart(k) => write!(
                f,
                "for loop start expression of type {}, expected integer",
                kind_to_string(k)
            ),
            Self::NonIntegerEnd(k) => write!(
                f,
                "for loop end expression of type {}, expected integer",
                kind_to_string(k)
            ),
        }
    }
}
impl fmt::Display for semantic_error::ReturnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReturnOutsideFunction => write!(
                f,
                "return statement is not allowd in main body, only in function declaration"
            ),
            Self::MismatchedReturnType(correct, given) => write!(
                f,
                "return statement type: {}, but {} was expected",
                kind_to_string(correct),
                kind_to_string(given)
            ),
        }
    }
}
impl<'a> fmt::Display for semantic_error::ArgumentCountError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "function: {} expected {} args, but {} are used in function call",
            self.func_decl.id,
            self.func_decl.params.len(),
            self.func_call.args.len()
        )
    }
}
impl<'a> fmt::Display for semantic_error::MismatchedArgumentType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "calling function {} argument expected type: {}, found {}",
            self.func_name,
            kind_to_string(&self.correct),
            kind_to_string(&self.given)
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
