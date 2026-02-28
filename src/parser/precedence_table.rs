use num_enum::IntoPrimitive;

use crate::lexer::tokens::TokenKind;

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum Precedence {
    Default,
    Assignment,
    Ternary,
    LogicalOr,
    LogicalAnd,
    Equals,
    Comparison,
    Additive,
    Multiplicative,
    Unary,
    Exponent,
    Call,
    Grouping,
}

impl Precedence {
    pub fn get_precedence(token_kind: TokenKind) -> Option<Self> {
        match token_kind {
            TokenKind::Plus | TokenKind::Minus => Some(Self::Additive),
            TokenKind::Multiply | TokenKind::Divide => Some(Self::Multiplicative),
            TokenKind::LeftParenthesis => Some(Self::Grouping),
            _ => None,
        }
    }
}
