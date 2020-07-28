use simpla_parser::syntax_tree;
use std::convert;
use std::fmt;

#[derive(PartialEq, Debug)]
pub enum SemanticError<'a> {
    NameRidefinition(NameRidefinition),
    VoidVariableDeclaration(VoidVariableDeclaration<'a>),
    MismatchedOperationTypes(MismatchedTypes),
    IncoherentOperation(IncoherentOperation),
    CastError(CastError),
    NonBooleanCondition(NonBooleanCondition),
    MismatchedConditionalExpression(MismatchedTypes),
    UnknownFunction(&'a str),
    UnknownVariable(&'a str),
    MismatchedUnary(MismatchedUnary),
    ArgumentCountError(ArgumentCountError<'a>),
    MismatchedArgumentType(MismatchedArgumentType<'a>),
    MismatchedAssignment(MismatchedAssignment<'a>),
    BreakOutsideLoop,
    ForLoopError(ForLoopError<'a>),
    ReturnError(ReturnError),
}

impl<'a> convert::From<SemanticError<'a>> for String {
    fn from(err: SemanticError) -> String {
        format!("{}", err)
    }
}

impl<'a> fmt::Display for SemanticError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (decr, msg) = match self {
            Self::NameRidefinition(err) => ("name error:", format!("{}", err)),
            Self::VoidVariableDeclaration(err) => ("void declaration error:", format!("{}", err)),
            Self::MismatchedOperationTypes(err) => ("mismatched operation error:", format!("{}", err)),
            Self::IncoherentOperation(err) => ("incoherent operation error:", format!("{}", err)),
            Self::CastError(err) => ("cast error:", format!("{}", err)),
            Self::NonBooleanCondition(err) => ("condition error:", format!("{}", err)),
            Self::MismatchedConditionalExpression(err) => ("conditional expression error:", format!("{}", err)),
            Self::UnknownFunction(err) => ("unknown function error:", format!("{}", err)),
            Self::UnknownVariable(err) => ("unknonw variable error:", format!("{}", err)),
            Self::MismatchedUnary(err) => ("negation error:", format!("{}", err)),
            Self::ArgumentCountError(err) => ("argument count error:", format!("{}", err)),
            Self::MismatchedArgumentType(err) => ("argument type error:", format!("{}", err)),
            Self::MismatchedAssignment(err) => ("assignment error:", format!("{}", err)),
            Self::BreakOutsideLoop => ("break error:", format!("break outside loop")),
            Self::ForLoopError(err) => ("for loop error:", format!("{}", err)),
            Self::ReturnError(err) => ("return error:", format!("{}", err)),    
        };
        write!(f, "{} {}", decr, msg)
    }
}

#[derive(Debug, PartialEq)]
pub struct NameRidefinition {
    pub name: String,
    pub original: Ridefinition,
    pub new: Ridefinition,
}

impl NameRidefinition {
    pub fn new(name: String, original: Ridefinition, new: Ridefinition) -> Self {
        Self {
            name,
            original,
            new,
        }
    }
}

impl fmt::Display for NameRidefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} defined multiple times, originally: {}, redefined: {}",
            self.name, self.original, self.new
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum Ridefinition {
    Function,
    Variable,
}

impl fmt::Display for Ridefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Variable => write!(f, "variable"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct VoidVariableDeclaration<'a> {
    pub names: &'a syntax_tree::VarDecl,
}

impl<'a> VoidVariableDeclaration<'a> {
    pub fn new(names: &'a syntax_tree::VarDecl) -> Self {
        Self { names }
    }
}

impl<'a> fmt::Display for VoidVariableDeclaration<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp = self.names.id_list.join(", ");
        write!(
            f,
            "Variables: [{}] defined as type void: only function can have type void",
            tmp
        )
    }
}

#[derive(PartialEq, Debug)]
pub struct MismatchedTypes {
    pub left: syntax_tree::Kind,
    pub right: syntax_tree::Kind,
}

impl MismatchedTypes {
    pub fn new(left: syntax_tree::Kind, right: syntax_tree::Kind) -> Self {
        Self { left, right }
    }
}

impl fmt::Display for MismatchedTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "left type: {} right type: {}",
            kind_to_string(&self.left),
            kind_to_string(&self.right)
        )
    }
}

#[derive(PartialEq, Debug)]
pub struct IncoherentOperation {
    pub var_kind: syntax_tree::Kind,
    pub operator: syntax_tree::Operator,
}

impl IncoherentOperation {
    pub fn new(var_kind: syntax_tree::Kind, operator: syntax_tree::Operator) -> Self {
        Self { var_kind, operator }
    }
}

impl fmt::Display for IncoherentOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "cannot apply operator {} to type {}",
            operator_to_string(&self.operator),
            kind_to_string(&self.var_kind)
        )
    }
}

#[derive(PartialEq, Debug)]
pub enum CastError {
    ToInt(syntax_tree::Kind),
    ToReal(syntax_tree::Kind),
}

impl fmt::Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ToInt(k) => write!(f, "cannot cast {} into integer", kind_to_string(k)),
            Self::ToReal(k) => write!(f, "cannot cast {} into real", kind_to_string(k)),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum NonBooleanCondition {
    IfStat(syntax_tree::Kind),
    WhileStat(syntax_tree::Kind),
    CondStat(syntax_tree::Kind),
}

impl fmt::Display for NonBooleanCondition {
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

#[derive(PartialEq, Debug)]
pub enum MismatchedUnary {
    Logic(syntax_tree::Kind),
    Numeric(syntax_tree::Kind),
}

impl fmt::Display for MismatchedUnary {
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

#[derive(PartialEq, Debug)]
pub struct MismatchedAssignment<'a> {
    pub name: &'a str,
    pub correct: syntax_tree::Kind,
    pub given: syntax_tree::Kind,
}

impl<'a> MismatchedAssignment<'a> {
    pub fn new(name: &'a str, correct: syntax_tree::Kind, given: syntax_tree::Kind) -> Self {
        Self {
            name,
            correct,
            given,
        }
    }
}

impl<'a> fmt::Display for MismatchedAssignment<'a> {
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

#[derive(PartialEq, Debug)]
pub enum ForLoopError<'a> {
    NonIntegerCount(syntax_tree::Kind),
    NonIntegerStart(syntax_tree::Kind),
    NonIntegerEnd(syntax_tree::Kind),
    CountVariableAssignment(&'a str),
}

impl<'a> fmt::Display for ForLoopError<'a> {
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

#[derive(PartialEq, Debug)]
pub enum ReturnError {
    ReturnOutsideFunction,
    //MissingReturn(&'a str),
    MismatchedReturnType(syntax_tree::Kind, syntax_tree::Kind),
}

impl fmt::Display for ReturnError {
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

#[derive(Debug, PartialEq)]
pub struct ArgumentCountError<'a> {
    pub func_decl: &'a syntax_tree::FuncDecl,
    pub func_call: &'a syntax_tree::FuncCall,
}

impl<'a> ArgumentCountError<'a> {
    pub fn new(func_decl: &'a syntax_tree::FuncDecl, func_call: &'a syntax_tree::FuncCall) -> Self {
        Self {
            func_decl,
            func_call,
        }
    }
}

impl<'a> fmt::Display for ArgumentCountError<'a> {
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

#[derive(Debug, PartialEq)]
pub struct MismatchedArgumentType<'a> {
    pub func_name: &'a str,
    pub correct: syntax_tree::Kind,
    pub given: syntax_tree::Kind,
}

impl<'a> MismatchedArgumentType<'a> {
    pub fn new(func_name: &'a str, correct: syntax_tree::Kind, given: syntax_tree::Kind) -> Self {
        Self {
            func_name,
            correct,
            given,
        }
    }
}

impl<'a> fmt::Display for MismatchedArgumentType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "calling function {} argument expected type: {}, found {}", self.func_name, kind_to_string(&self.correct), kind_to_string(&self.given))
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
