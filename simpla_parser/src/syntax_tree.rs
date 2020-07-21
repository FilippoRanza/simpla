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
pub enum Kind {
    Int,
    Real,
    Str,
    Bool,
    Void,
}

#[derive(PartialEq, Debug)]
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
}

impl VarDecl {
    pub fn new(id_list: IdList, kind: Kind) -> Self {
        Self { id_list, kind }
    }
}

#[derive(PartialEq, Debug)]
pub struct ParamDecl {
    id: String,
    kind: Kind,
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
}

impl FuncDecl {
    pub fn new(
        id: String,
        params: ParamList,
        kind: Kind,
        vars: VarDeclList,
        body: StatList,
    ) -> Self {
        Self {
            id,
            params,
            kind,
            vars,
            body,
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
    id: String,
    expr: Expr,
}

impl AssignStat {
    pub fn new(id: String, expr: Expr) -> Self {
        Self { id, expr }
    }
}

#[derive(PartialEq, Debug)]
pub struct IfStat {
    pub cond: Expr,
    pub if_body: StatList,
    pub else_body: Option<StatList>,
}

impl IfStat {
    pub fn new(cond: Expr, if_body: StatList, else_body: Option<StatList>) -> Self {
        Self {
            cond,
            if_body,
            else_body,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct WhileStat {
    cond: Expr,
    body: StatList,
}

impl WhileStat {
    pub fn new(cond: Expr, body: StatList) -> Self {
        Self { cond, body }
    }
}

#[derive(PartialEq, Debug)]
pub struct ForStat {
    id: String,
    begin_expr: Expr,
    end_expr: Expr,
    body: StatList,
}

impl ForStat {
    pub fn new(id: String, begin_expr: Expr, end_expr: Expr, body: StatList) -> Self {
        Self {
            id,
            begin_expr,
            end_expr,
            body,
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
}

impl FuncCall {
    pub fn new(id: String, args: ExprList) -> Self {
        Self { id, args }
    }
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    Node(Box<Expr>, Operator, Box<Expr>),
    Factor(Factor),
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
    cond: Box<Expr>,
    true_stat: Box<Expr>,
    false_stat: Box<Expr>,
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
pub enum CastExpr {
    Integer(Box<Expr>),
    Real(Box<Expr>),
}

pub type StatList = Vec<Stat>;
pub type VarDeclList = Vec<VarDecl>;
pub type ParamList = Vec<ParamDecl>;
pub type IdList = Vec<String>;
pub type ExprList = Vec<Expr>;
pub type FuncDeclList = Vec<FuncDecl>;
