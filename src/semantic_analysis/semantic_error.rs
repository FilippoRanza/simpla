use simpla_parser::syntax_tree;

#[derive(PartialEq, Debug)]
pub enum SemanticError<'a> {
    NameRidefinition(NameRidefinition),
    VoidVariableDeclaration(VoidVariableDeclaration<'a>),
    MismatchedOperationTypes(MismatchedTypes<'a>),
    IncoherentOperation(IncoherentOperation<'a>),
    CastError(CastError<'a>),
    NonBooleanCondition(NonBooleanCondition<'a>),
    MismatchedConditionalExpression(MismatchedTypes<'a>),
    UnknownFunction(&'a str),
    UnknownVariable(&'a str),
    MismatchedUnary(MismatchedUnary<'a>),
    ArgumentCountError(ArgumentCountError<'a>),
    MismatchedArgumentType(MismatchedArgumentType<'a>),
    MismatchedAssignment(MismatchedAssignment<'a>),
    BreakOutsideLoop,
    ForLoopError(ForLoopError<'a>),
    ReturnError(ReturnError<'a>),
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
pub struct MismatchedTypes<'a> {
    pub left: syntax_tree::Kind,
    pub right: syntax_tree::Kind,
    pub loc: &'a syntax_tree::Location,
}

impl<'a> MismatchedTypes<'a> {
    pub fn new(
        left: syntax_tree::Kind,
        right: syntax_tree::Kind,
        loc: &'a syntax_tree::Location,
    ) -> Self {
        Self { left, right, loc }
    }
}

#[derive(PartialEq, Debug)]
pub struct IncoherentOperation<'a> {
    pub var_kind: syntax_tree::Kind,
    pub operator: syntax_tree::Operator,
    pub loc: &'a syntax_tree::Location,
}

impl<'a> IncoherentOperation<'a> {
    pub fn new(
        var_kind: syntax_tree::Kind,
        operator: syntax_tree::Operator,
        loc: &'a syntax_tree::Location,
    ) -> Self {
        Self {
            var_kind,
            operator,
            loc,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct CastError<'a> {
    pub loc: &'a syntax_tree::Location,
    pub error: CastErrorType,
}

impl<'a> CastError<'a> {
    pub fn new_to_int(loc: &'a syntax_tree::Location, kind: syntax_tree::Kind) -> Self {
        let error = CastErrorType::ToInt(kind);
        Self { loc, error }
    }

    pub fn new_to_real(loc: &'a syntax_tree::Location, kind: syntax_tree::Kind) -> Self {
        let error = CastErrorType::ToReal(kind);
        Self { loc, error }
    }
}

#[derive(PartialEq, Debug)]
pub enum CastErrorType {
    ToInt(syntax_tree::Kind),
    ToReal(syntax_tree::Kind),
}

#[derive(PartialEq, Debug)]
pub struct NonBooleanCondition<'a> {
    pub loc: &'a syntax_tree::Location,
    pub error: NonBooleanConditionType,
}

impl<'a> NonBooleanCondition<'a> {
    pub fn new_if_stat(loc: &'a syntax_tree::Location, kind: syntax_tree::Kind) -> Self {
        let error = NonBooleanConditionType::IfStat(kind);
        Self { loc, error }
    }

    pub fn new_while_stat(loc: &'a syntax_tree::Location, kind: syntax_tree::Kind) -> Self {
        let error = NonBooleanConditionType::WhileStat(kind);
        Self { loc, error }
    }

    pub fn new_cond_stat(loc: &'a syntax_tree::Location, kind: syntax_tree::Kind) -> Self {
        let error = NonBooleanConditionType::CondStat(kind);
        Self { loc, error }
    }
}

#[derive(PartialEq, Debug)]
pub enum NonBooleanConditionType {
    IfStat(syntax_tree::Kind),
    WhileStat(syntax_tree::Kind),
    CondStat(syntax_tree::Kind),
}

#[derive(PartialEq, Debug)]
pub struct MismatchedUnary<'a> {
    pub loc: &'a syntax_tree::Location,
    pub error: MismatchedUnaryType,
}

impl<'a> MismatchedUnary<'a> {
    pub fn new_logic(loc: &'a syntax_tree::Location, kind: syntax_tree::Kind) -> Self {
        let error = MismatchedUnaryType::Logic(kind);
        Self { loc, error }
    }

    pub fn new_numeric(loc: &'a syntax_tree::Location, kind: syntax_tree::Kind) -> Self {
        let error = MismatchedUnaryType::Numeric(kind);
        Self { loc, error }
    }
}

#[derive(PartialEq, Debug)]
pub enum MismatchedUnaryType {
    Logic(syntax_tree::Kind),
    Numeric(syntax_tree::Kind),
}

#[derive(PartialEq, Debug)]
pub struct MismatchedAssignment<'a> {
    pub name: &'a str,
    pub correct: syntax_tree::Kind,
    pub given: syntax_tree::Kind,
    pub loc: &'a syntax_tree::Location,
}

impl<'a> MismatchedAssignment<'a> {
    pub fn new(
        name: &'a str,
        correct: syntax_tree::Kind,
        given: syntax_tree::Kind,
        loc: &'a syntax_tree::Location,
    ) -> Self {
        Self {
            name,
            correct,
            given,
            loc,
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
pub struct ReturnError<'a> {
    pub loc: &'a syntax_tree::Location,
    pub error: ReturnErrorType
}

impl<'a> ReturnError<'a> {
    pub fn new_return_outside_function(loc: &'a syntax_tree::Location) -> Self {
        Self {
            loc, 
            error: ReturnErrorType::ReturnOutsideFunction
        }
    }

    pub fn new_mismatched_type(loc: &'a syntax_tree::Location, decl: syntax_tree::Kind, given: syntax_tree::Kind) -> Self {
        Self {
            loc, 
            error: ReturnErrorType::MismatchedReturnType(decl, given)
        }
    }
    
}


#[derive(PartialEq, Debug)]
pub enum ReturnErrorType {
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
    pub func: &'a syntax_tree::FuncDecl,
    pub correct: syntax_tree::Kind,
    pub given: syntax_tree::Kind,
    pub index: usize,
    pub loc: &'a syntax_tree::Location,
}

impl<'a> MismatchedArgumentType<'a> {
    pub fn new(
        func: &'a syntax_tree::FuncDecl,
        correct: syntax_tree::Kind,
        given: syntax_tree::Kind,
        index: usize,
        loc: &'a syntax_tree::Location,
    ) -> Self {
        Self {
            func,
            correct,
            given,
            index,
            loc,
        }
    }
}
