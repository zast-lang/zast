use crate::lexer::tokens::Span;

pub struct ZastProgram {
    pub body: Vec<Statement>,
}

pub type Expression = Spanned<Expr>;
pub enum Expr {
    IntegerLiteral(i64),
    FloatLiteral(f64),
}

pub type Statement = Spanned<Stmt>;
pub enum Stmt {
    Expression { expression: Expression },
}

impl Stmt {
    pub fn spanned(self, span: Span) -> Spanned<Self> {
        Spanned { node: self, span }
    }
}

impl Expr {
    pub fn spanned(self, span: Span) -> Spanned<Self> {
        Spanned { node: self, span }
    }
}
pub struct Spanned<T> {
    node: T,
    span: Span,
}
