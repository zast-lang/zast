use crate::{
    lexer::tokens::{Span, TokenKind},
    types::{annotated_type::AnnotatedType, return_type::ReturnType},
};

#[derive(Debug)]
pub struct ZastProgram {
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct FunctionParameter {
    pub name: String,
    pub annotated_type: AnnotatedType,
}

pub type Expression = Spanned<Expr>;
#[derive(Debug)]
pub enum Expr {
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Identifier(String),
    Address(Box<Expression>),
    Dereference(Box<Expression>),
    BinaryExpression {
        left: Box<Expression>,
        operator: TokenKind,
        right: Box<Expression>,
    },
}

pub type Statement = Spanned<Stmt>;
#[derive(Debug)]
pub enum Stmt {
    FunctionDeclaration {
        name: String,
        parameters: Vec<FunctionParameter>,
        return_type: ReturnType,
        body: Box<Statement>, // Block Statement
    },
    BlockStatement {
        statements: Vec<Box<Statement>>,
    },
    Expression {
        expression: Expression,
    },
    VariableDeclaration {
        mutable: bool,
        identifier: String,
        annotated_type: AnnotatedType,
        value: Expression,
    },
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
