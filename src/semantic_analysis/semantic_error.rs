use simpla_parser::syntax_tree;

#[derive(Debug)]
pub enum SemanticError<'a> {
    NameRidefinition {
        name: String,
        original: Ridefinition,
        new: Ridefinition,
    },
    VoidVariableDeclaration {
        names: &'a [String],
    },
    MismatchedOperationTypes(MismatchedTypes),
    IncoherentOperation(IncoherentOperation),
    CastError(CastError),
    NonBooleanCondition(NonBooleanCondition),
    MismatchedConditionalExpression(MismatchedTypes),
    UnknownFunction(&'a str),
    UnknownVariable(&'a str),
    MismatchedUnary(MismatchedUnary),
    ArgumentCountError {
        func_name: &'a str,
        correct: usize,
        given: usize,
    },
    MismatchedArgumentType {
        func_name: &'a str,
        correct: syntax_tree::Kind,
        given: syntax_tree::Kind,
    },
    InnerError,
}

#[derive(Debug, PartialEq)]
pub enum Ridefinition {
    Function,
    Variable,
}

#[derive(Debug)]
pub struct MismatchedTypes {
    pub left: syntax_tree::Kind,
    pub right: syntax_tree::Kind,
}

impl MismatchedTypes {
    pub fn new(left: syntax_tree::Kind, right: syntax_tree::Kind) -> Self {
        Self { left, right }
    }
}

#[derive(Debug)]
pub struct IncoherentOperation {
    pub var_kind: syntax_tree::Kind,
    pub operator: syntax_tree::Operator,
}

impl IncoherentOperation {
    pub fn new(var_kind: syntax_tree::Kind, operator: syntax_tree::Operator) -> Self {
        Self { var_kind, operator }
    }
}

#[derive(Debug)]
pub enum CastError {
    ToInt(syntax_tree::Kind),
    ToReal(syntax_tree::Kind),
}

#[derive(Debug)]
pub enum NonBooleanCondition {
    IfStat(syntax_tree::Kind),
    WhileStat(syntax_tree::Kind),
    CondStat(syntax_tree::Kind),
}

#[derive(Debug)]
pub enum MismatchedUnary {
    Logic(syntax_tree::Kind),
    Numeric(syntax_tree::Kind),
}
