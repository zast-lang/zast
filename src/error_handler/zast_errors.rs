use crate::lexer::tokens::{Span, TokenKind};

pub enum ZastError {
    UnexpectedToken { span: Span, token_kind: TokenKind },
    IllegalToken { span: Span, token_lexeme: String },
}

impl ZastError {
    pub fn get_span(&self) -> Span {
        match self {
            Self::UnexpectedToken { span, .. } => *span,
            Self::IllegalToken { span, .. } => *span,
        }
    }

    pub fn get_error_msg(&self) -> String {
        match self {
            Self::UnexpectedToken { token_kind, .. } => {
                format!("Unexpected token found '{:?}'", token_kind)
            }
            Self::IllegalToken { token_lexeme, .. } => {
                format!("Illegal token found '{}'", token_lexeme)
            }
        }
    }
}
