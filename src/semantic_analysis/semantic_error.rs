use simpla_parser::syntax_tree;

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

#[derive(Debug, PartialEq)]
pub enum Ridefinition {
    Function(syntax_tree::Location),
    Variable(syntax_tree::Location),
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

#[derive(PartialEq, Debug)]
pub enum CastError {
    ToInt(syntax_tree::Kind),
    ToReal(syntax_tree::Kind),
}

#[derive(PartialEq, Debug)]
pub enum NonBooleanCondition {
    IfStat(syntax_tree::Kind),
    WhileStat(syntax_tree::Kind),
    CondStat(syntax_tree::Kind),
}

#[derive(PartialEq, Debug)]
pub enum MismatchedUnary {
    Logic(syntax_tree::Kind),
    Numeric(syntax_tree::Kind),
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

#[derive(PartialEq, Debug)]
pub struct ForLoopError<'a> {
    pub loc: &'a syntax_tree::Location,
    pub error: ForLoopErrorType<'a>,
}

impl<'a> ForLoopError<'a> {
    pub fn new_non_integer_count(loc: &'a syntax_tree::Location, kind: syntax_tree::Kind) -> Self {
        let error = ForLoopErrorType::NonIntegerCount(kind);
        Self { loc, error }
    }

    pub fn new_non_integer_start(loc: &'a syntax_tree::Location, kind: syntax_tree::Kind) -> Self {
        let error = ForLoopErrorType::NonIntegerStart(kind);
        Self { loc, error }
    }

    pub fn new_non_integer_end(loc: &'a syntax_tree::Location, kind: syntax_tree::Kind) -> Self {
        let error = ForLoopErrorType::NonIntegerEnd(kind);
        Self { loc, error }
    }

    pub fn new_count_variable_assignment(loc: &'a syntax_tree::Location, name: &'a str) -> Self {
        let error = ForLoopErrorType::CountVariableAssignment(name);
        Self { loc, error }
    }
}

#[derive(PartialEq, Debug)]
pub enum ForLoopErrorType<'a> {
    NonIntegerCount(syntax_tree::Kind),
    NonIntegerStart(syntax_tree::Kind),
    NonIntegerEnd(syntax_tree::Kind),
    CountVariableAssignment(&'a str),
}

#[derive(PartialEq, Debug)]
pub enum ReturnError {
    ReturnOutsideFunction,
    //MissingReturn(&'a str),
    MismatchedReturnType(syntax_tree::Kind, syntax_tree::Kind),
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
