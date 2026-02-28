use crate::{
    ast::{Expr, Expression},
    error_handler::zast_errors::{Expected, ZastError},
    lexer::tokens::{Span, TokenKind},
    parser::{ZastParser, precedence_table::Precedence},
};

impl ZastParser {
    /// Parses an expression using Pratt (top-down operator precedence) parsing.
    ///
    /// Begins by dispatching to the NUD function for the current token. Then
    /// repeatedly consumes LED (infix) operators whose precedence exceeds the
    /// given minimum, building up the left-hand side of the expression.
    ///
    /// # Arguments
    ///
    /// * `precedence` - The minimum precedence level. Operators at or below
    ///   this level will not be consumed.
    ///
    /// # Returns
    ///
    /// `Some(Expression)` on success, or `None` if no NUD function is registered
    /// for the current token (in which case an [`ZastError::UnexpectedToken`] is emitted).
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

    /// Parses a unary dereference expression, e.g. `*ptr`.
    ///
    /// Consumes the `*` token and parses the operand at [`Precedence::Unary`]
    /// so that only the immediate right-hand primary is consumed.
    pub fn parse_deref_expr(&mut self) -> Option<Expression> {
        let op_span = self.current_token().span;
        self.advance(); // eat '*'

        let operand = self.try_parse_expr(Precedence::Unary)?;
        let full_span = Span {
            col_start: op_span.col_start,
            col_end: operand.span.col_end,
            ln_start: op_span.ln_start,
            ln_end: operand.span.ln_end,
        };

        Some(Expr::Dereference(Box::new(operand)).spanned(full_span))
    }

    /// Parses a unary address-of expression, e.g. `&x`.
    ///
    /// Consumes the `&` token and parses the operand at [`Precedence::Unary`]
    /// so that only the immediate right-hand primary is consumed.
    pub fn parse_addr_expr(&mut self) -> Option<Expression> {
        let op_span = self.current_token().span;
        self.advance(); // eat '&'

        let operand = self.try_parse_expr(Precedence::Unary)?;
        let full_span = Span {
            col_start: op_span.col_start,
            col_end: operand.span.col_end,
            ln_start: op_span.ln_start,
            ln_end: operand.span.ln_end,
        };

        Some(Expr::Address(Box::new(operand)).spanned(full_span))
    }

    /// Parses an integer literal token into an [`Expr::IntegerLiteral`].
    ///
    /// # Panics
    ///
    /// Panics if the current token's literal is not an [`Literal::IntegerValue`].
    /// This should not occur under normal operation since this function is only
    /// dispatched for [`TokenKind::Integer`] tokens.
    pub fn parse_integer_literal(&mut self) -> Option<Expression> {
        let span = self.current_token().span;
        let expr = Expr::IntegerLiteral(self.current_token().literal.get_int().unwrap());
        self.advance();
        Some(expr.spanned(span))
    }

    /// Parses a float literal token into an [`Expr::FloatLiteral`].
    ///
    /// # Panics
    ///
    /// Panics if the current token's literal is not a [`Literal::FloatValue`].
    /// This should not occur under normal operation since this function is only
    /// dispatched for [`TokenKind::Float`] tokens.
    pub fn parse_float_literal(&mut self) -> Option<Expression> {
        let span = self.current_token().span;
        let expr = Expr::FloatLiteral(self.current_token().literal.get_float().unwrap());
        self.advance();
        Some(expr.spanned(span))
    }

    /// Parses an identifier token into an [`Expr::Identifier`].
    ///
    /// # Panics
    ///
    /// Panics if the current token's literal is not a [`Literal::Identifier`].
    /// This should not occur under normal operation since this function is only
    /// dispatched for [`TokenKind::Identifier`] tokens.
    pub fn parse_identifier_literal(&mut self) -> Option<Expression> {
        let span = self.current_token().span;
        let expr = Expr::Identifier(self.current_token().literal.get_identifier().unwrap());
        self.advance();
        Some(expr.spanned(span))
    }

    /// Parses a binary infix expression, e.g. `a + b`, `x * y`.
    ///
    /// Called as a LED function with the left-hand expression already parsed.
    /// Consumes the operator token and recursively parses the right-hand side
    /// at the operator's own precedence level.
    ///
    /// # Arguments
    ///
    /// * `left` - The already-parsed left-hand expression.
    pub fn parse_binary_expr(&mut self, left: Expression) -> Option<Expression> {
        let op = self.current_token().kind;
        let left_span = left.span;
        self.advance(); // eat operator

        let right =
            self.try_parse_expr(Precedence::get_precedence(op).unwrap_or(Precedence::Default))?;
        let right_span = right.span;

        let full_span = Span {
            ln_start: left_span.ln_start,
            ln_end: right_span.ln_end,
            col_start: left_span.col_start,
            col_end: right_span.col_end,
        };

        Some(
            Expr::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            }
            .spanned(full_span),
        )
    }

    /// Parses a parenthesized grouping expression, e.g. `(a + b)`.
    ///
    /// Consumes the opening `(`, parses the inner expression at default
    /// precedence, then expects a closing `)`. The grouping itself produces
    /// no AST node — the inner expression is returned directly.
    pub fn parse_grouping_expression(&mut self) -> Option<Expression> {
        self.advance(); // eat '('
        let expr = self.try_parse_expr(Precedence::Default)?;

        if !self.expect(vec![Expected::Token(TokenKind::RightParenthesis)]) {
            return None;
        }

        Some(expr)
    }
}
