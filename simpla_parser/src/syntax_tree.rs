#[derive(PartialEq, Debug)]
pub struct Program {
    pub global_vars: VarDeclList,
    pub functions: FuncDeclList,
    pub body: StatList,
}

impl Program {
    pub fn new(global_vars: VarDeclList, functions: FuncDeclList, body: StatList) -> Self {
        Self {
            global_vars,
            functions,
            body,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Location {
    pub begin: usize,
    pub end: usize,
}

impl Location {
    pub fn new(begin: usize, end: usize) -> Self {
        Self { begin, end }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Kind {
    Int,
    Real,
    Str,
    Bool,
    Void,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Operator {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
}

#[derive(PartialEq, Debug)]
pub struct VarDecl {
    pub id_list: IdList,
    pub kind: Kind,
    pub loc: Location,
}

impl VarDecl {
    pub fn new(id_list: IdList, kind: Kind, begin: usize, end: usize) -> Self {
        Self {
            id_list,
            kind,
            loc: Location::new(begin, end),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ParamDecl {
    pub id: String,
    pub kind: Kind,
}

impl ParamDecl {
    pub fn new(id: String, kind: Kind) -> Self {
        Self { id, kind }
    }
}

#[derive(PartialEq, Debug)]
pub struct FuncDecl {
    pub id: String,
    pub kind: Kind,
    pub params: ParamList,
    pub vars: VarDeclList,
    pub body: StatList,
    pub loc: Location,
}

impl FuncDecl {
    pub fn new(
        id: String,
        params: ParamList,
        kind: Kind,
        vars: VarDeclList,
        body: StatList,
        begin: usize,
        end: usize,
    ) -> Self {
        Self {
            id,
            params,
            kind,
            vars,
            body,
            loc: Location::new(begin, end),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Stat {
    AssignStat(AssignStat),
    IfStat(IfStat),
    WhileStat(WhileStat),
    ForStat(ForStat),
    ReturnStat(Option<Expr>),
    ReadStat(IdList),
    WriteStat(WriteStat),
    FuncCall(FuncCall),
    Break,
}

#[derive(PartialEq, Debug)]
pub struct AssignStat {
    pub id: String,
    pub expr: Expr,
    pub loc: Location,
}

impl AssignStat {
    pub fn new(id: String, expr: Expr, begin: usize, end: usize) -> Self {
        Self {
            id,
            expr,
            loc: Location::new(begin, end),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct IfStat {
    pub cond: Expr,
    pub if_body: StatList,
    pub else_body: Option<StatList>,
    pub loc: Location,
}

impl IfStat {
    pub fn new(
        cond: Expr,
        if_body: StatList,
        else_body: Option<StatList>,
        begin: usize,
        end: usize,
    ) -> Self {
        Self {
            cond,
            if_body,
            else_body,
            loc: Location::new(begin, end),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct WhileStat {
    pub cond: Expr,
    pub body: StatList,
    pub loc: Location,
}

impl WhileStat {
    pub fn new(cond: Expr, body: StatList, begin: usize, end: usize) -> Self {
        Self {
            cond,
            body,
            loc: Location::new(begin, end),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ForStat {
    pub id: String,
    pub begin_expr: Expr,
    pub end_expr: Expr,
    pub body: StatList,
    pub loc: Location,
}

impl ForStat {
    pub fn new(
        id: String,
        begin_expr: Expr,
        end_expr: Expr,
        body: StatList,
        begin: usize,
        end: usize,
    ) -> Self {
        Self {
            id,
            begin_expr,
            end_expr,
            body,
            loc: Location::new(begin, end),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum WriteStat {
    WriteLine(ExprList),
    Write(ExprList),
}

#[derive(PartialEq, Debug)]
pub struct FuncCall {
    pub id: String,
    pub args: ExprList,
    pub loc: Location,
}

impl FuncCall {
    pub fn new(id: String, args: ExprList, begin: usize, end: usize) -> Self {
        Self {
            id,
            args,
            loc: Location::new(begin, end),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Expr {
    pub loc: Location,
    pub expr: ExprTree,
}

impl Expr {
    pub fn new(expr: ExprTree, begin: usize, end: usize) -> Self {
        Self {
            expr,
            loc: Location::new(begin, end),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ExprTree {
    Node(Box<Expr>, Operator, Box<Expr>),
    Factor(Factor),
}

#[derive(PartialEq, Debug)]
pub struct FactorWrapper {
    pub loc: Location,
    pub fact: Factor,
}

#[derive(PartialEq, Debug)]
pub enum Factor {
    Id(String),
    UnaryOp(UnaryOp),
    CondExpr(CondExpr),
    CastExpr(CastExpr),
    FuncCall(FuncCall),
    Const(Const),
    HighPrecedence(Box<Expr>),
}

#[derive(PartialEq, Debug)]
pub enum Const {
    IntConst(i32),
    RealConst(f64),
    StrConst(String),
    BoolConst(bool),
}

#[derive(PartialEq, Debug)]
pub enum UnaryOp {
    Negate(Box<Factor>),
    Minus(Box<Factor>),
}

#[derive(PartialEq, Debug)]
pub struct CondExpr {
    pub cond: Box<Expr>,
    pub true_stat: Box<Expr>,
    pub false_stat: Box<Expr>,
}

impl CondExpr {
    pub fn new(cond: Expr, true_stat: Expr, false_stat: Expr) -> Self {
        Self {
            cond: Box::new(cond),
            true_stat: Box::new(true_stat),
            false_stat: Box::new(false_stat),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct CastExpr {
    pub loc: Location,
    pub expr: CastExprType,
}

impl CastExpr {
    pub fn new_to_integer(expr: Box<Expr>, begin: usize, end: usize) -> Self {
        Self {
            expr: CastExprType::Integer(expr),
            loc: Location::new(begin, end),
        }
    }

    pub fn new_to_real(expr: Box<Expr>, begin: usize, end: usize) -> Self {
        Self {
            expr: CastExprType::Real(expr),
            loc: Location::new(begin, end),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum CastExprType {
    Integer(Box<Expr>),
    Real(Box<Expr>),
}

pub type StatList = Vec<Stat>;
pub type VarDeclList = Vec<VarDecl>;
pub type ParamList = Vec<ParamDecl>;
pub type IdList = Vec<String>;
pub type ExprList = Vec<Expr>;
pub type FuncDeclList = Vec<FuncDecl>;
