pub struct Program {
    global_vars: VarDeclList,
    functions: FuncDeclList,
    body: StatList,
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

pub enum Kind {
    Int,
    Real,
    Str,
    Bool,
    Void,
}

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

pub struct VarDecl {
    id_list: IdList,
    kind: Kind,
}

impl VarDecl {
    pub fn new(id_list: IdList, kind: Kind) -> Self {
        Self { id_list, kind }
    }
}

pub struct ParamDecl {
    id: String,
    kind: Kind,
}

impl ParamDecl {
    pub fn new(id: String, kind: Kind) -> Self {
        Self { id, kind }
    }
}

pub struct FuncDecl {
    id: String,
    kind: Kind,
    params: ParamList,
    vars: VarDeclList,
    body: StatList,
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

pub struct AssignStat {
    id: String,
    expr: Expr,
}

impl AssignStat {
    pub fn new(id: String, expr: Expr) -> Self {
        Self { id, expr }
    }
}

pub struct IfStat {
    cond: Expr,
    if_body: StatList,
    else_body: Option<StatList>,
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

pub struct WhileStat {
    cond: Expr,
    body: StatList,
}

impl WhileStat {
    pub fn new(cond: Expr, body: StatList) -> Self {
        Self { cond, body }
    }
}

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

pub enum WriteStat {
    WriteLine(ExprList),
    Write(ExprList),
}

pub struct FuncCall {
    id: String,
    args: ExprList,
}

impl FuncCall {
    pub fn new(id: String, args: ExprList) -> Self {
        Self { id, args }
    }
}

pub enum Expr {
    Node(Box<Expr>, Operator, Box<Expr>),
    Factor(Factor),
}

pub enum Factor {
    Id(String),
    UnaryOp(UnaryOp),
    CondExpr(CondExpr),
    CastExpr(CastExpr),
    FuncCall(FuncCall),
    Const(Const),
}

pub enum Const {
    IntConst(i32),
    RealConst(f64),
    StrConst(String),
    BoolConst(bool),
}

pub enum UnaryOp {
    Negate(Box<Factor>),
    Minus(Box<Factor>),
}

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
