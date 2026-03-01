use core::fmt;

use crate::lexer::tokens::{Span, TokenKind};

#[derive(Debug)]
pub enum ZastError {
    // Parsing
    UnexpectedToken {
        span: Span,
        token_kind: TokenKind,
    },
    ExpectedToken {
        span: Span,
        expected_tokens: Vec<Expected>,
        found_token: TokenKind,
    },
    IllegalToken {
        span: Span,
        token_lexeme: String,
    },

    // Sema
    VariableRedeclaration {
        span: Span,
        variable_name: String,
        original_span: Span,
    },
    FunctionRedeclaration {
        span: Span,
        fn_name: String,
        original_span: Span,
    },
}

#[derive(Debug)]
pub enum Expected {
    Token(TokenKind),
    Concept(&'static str), // "type annotation", "expression"
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Token(kind) => write!(f, "'{:?}'", kind), // quoted
            Self::Concept(s) => write!(f, "{}", s),         // unquoted
        }
    }
}
