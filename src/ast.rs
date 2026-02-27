use crate::lexer::tokens::Span;

#[derive(Debug)]
pub struct ZastProgram {
    pub body: Vec<Statement>,
}

pub type Expression = Spanned<Expr>;
#[derive(Debug)]
pub enum Expr {
    IntegerLiteral(i64),
    FloatLiteral(f64),
}

pub type Statement = Spanned<Stmt>;
#[derive(Debug)]
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
#[derive(Debug)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}
