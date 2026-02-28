use crate::{
    ast::{Expr, Expression},
    error_handler::zast_errors::{Expected, ZastError},
    lexer::tokens::{Span, TokenKind},
    parser::{ZastParser, precedence_table::Precedence},
};

impl ZastParser {
    pub fn try_parse_expr(&mut self, precedence: Precedence) -> Option<Expression> {
        let current_tok = self.current_token();
        let prec: u8 = precedence.into();

        let nud_fn = self.nud_lookup.get(&current_tok.kind).cloned();

        if let Some(left_fn) = nud_fn {
            let mut left = left_fn(self)?;

            while !self.is_at_eof() {
                let next_prec = self.current_token_precedence();
                if prec >= next_prec {
                    break;
                }

                let led_fn = self.led_lookup.get(&self.current_token_kind()).cloned();
                if let Some(right_fn) = led_fn {
                    left = right_fn(self, left)?;
                } else {
                    break;
                }
            }

            return Some(left);
        }

        self.throw_error(ZastError::UnexpectedToken {
            span: current_tok.span,
            token_kind: current_tok.kind,
        });
        None
    }

    pub fn parse_integer_literal(&mut self) -> Option<Expression> {
        let current_tok_span = self.current_token().span;
        let current_tok_literal = self.current_token().literal.clone();
        let expr = Expr::IntegerLiteral(current_tok_literal.get_int().unwrap());

        self.advance();
        Some(expr.spanned(current_tok_span))
    }

    pub fn parse_float_literal(&mut self) -> Option<Expression> {
        let current_tok_span = self.current_token().span;
        let current_tok_literal = self.current_token().literal.clone();
        let expr = Expr::FloatLiteral(current_tok_literal.get_float().unwrap());

        self.advance();
        Some(expr.spanned(current_tok_span))
    }

    pub fn parse_identifier_literal(&mut self) -> Option<Expression> {
        let current_tok_span = self.current_token().span;
        let current_tok_literal = self.current_token().literal.clone();
        let expr = Expr::Identifier(current_tok_literal.get_identifier().unwrap());

        self.advance();
        Some(expr.spanned(current_tok_span))
    }

    pub fn parse_binary_expr(&mut self, left: Expression) -> Option<Expression> {
        let op = self.current_token().kind;
        let left_span = left.span;
        self.advance();
        let right =
            self.try_parse_expr(Precedence::get_precedence(op).unwrap_or(Precedence::Default))?;
        let right_span = right.span;
        let expr = Expr::BinaryExpression {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
        };

        let full_span = Span {
            ln_start: left_span.ln_start,
            ln_end: right_span.ln_end,
            col_start: left_span.col_start,
            col_end: right_span.col_end,
        };
        Some(expr.spanned(full_span))
    }

    pub fn parse_grouping_expression(&mut self) -> Option<Expression> {
        self.advance(); // consume '('
        let expr = self.try_parse_expr(Precedence::Default)?;

        if !self.expect(vec![Expected::Token(TokenKind::RightParenthesis)]) {
            return None;
        }

        Some(expr)
    }
}
