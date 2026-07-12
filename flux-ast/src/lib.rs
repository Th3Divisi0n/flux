use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn merge(a: &Span, b: &Span) -> Self {
        Self {
            start_line: a.start_line,
            start_col: a.start_col,
            end_line: b.end_line,
            end_col: b.end_col,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print {
        expr: Expr,
        span: Span,
    },
    Assign {
        name: String,
        expr: Expr,
        span: Span,
    },
    MemberAssign {
        object: Expr,
        member: String,
        expr: Expr,
        span: Span,
    },
    IndexAssign {
        object: Expr,
        index: Expr,
        expr: Expr,
        span: Span,
    },
    Expr {
        expr: Expr,
        span: Span,
    },
    FunctionDef {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
        span: Span,
    },
    Return {
        expr: Option<Expr>,
        span: Span,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        elif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    For {
        variable: String,
        iterable: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Pass {
        span: Span,
    },
    Import {
        module: String,
        alias: Option<String>,
        span: Span,
    },
    FromImport {
        module: String,
        names: Vec<(String, Option<String>)>,
        span: Span,
    },
    ClassDef {
        name: String,
        body: Vec<Stmt>,
        span: Span,
    },
    Try {
        body: Vec<Stmt>,
        handlers: Vec<ExceptHandler>,
        finally_body: Option<Vec<Stmt>>,
        span: Span,
    },
    Raise {
        expr: Option<Expr>,
        span: Span,
    },
    Ask {
        message: Expr,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExceptHandler {
    pub exception_type: Option<String>,
    pub binding: Option<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_hint: Option<String>,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        value: Literal,
        span: Span,
    },
    Variable {
        name: String,
        span: Span,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Member {
        object: Box<Expr>,
        member: String,
        span: Span,
    },
    Array {
        elements: Vec<Expr>,
        span: Span,
    },
    Dict {
        entries: Vec<(Expr, Expr)>,
        span: Span,
    },
    Lambda {
        params: Vec<Param>,
        body: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::FloorDiv => "//",
            BinaryOp::Mod => "%",
            BinaryOp::Pow => "**",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::LtEq => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "AND",
            BinaryOp::Or => "OR",
        };
        write!(f, "{s}")
    }
}
