use crate::{error_handler::zast_errors::ZastError, lexer::tokens::Span};

impl ZastError {
    pub fn get_span(&self) -> Span {
        match self {
            Self::UnexpectedToken { span, .. } => *span,
            Self::VariableRedeclaration { span, .. } => *span,
            Self::FunctionRedeclaration { span, .. } => *span,
            Self::ExpectedToken { span, .. } => *span,
            Self::IllegalToken { span, .. } => *span,
        }
    }
}
