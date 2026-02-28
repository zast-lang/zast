use crate::{
    ast::{Statement, Stmt},
    error_handler::zast_errors::Expected,
    lexer::tokens::{Span, TokenKind},
    parser::{ZastParser, precedence_table::Precedence},
};

impl ZastParser {
    pub fn try_parse_stmt(&mut self) -> Option<Statement> {
        if let Some(stmt_fn) = self.stmt_lookup.get(&self.current_token_kind()) {
            return stmt_fn(self);
        }

        let stmt_expr = self.try_parse_expr(Precedence::Default)?;
        let stmt_expr_span = stmt_expr.span;
        let stmt = Stmt::Expression {
            expression: stmt_expr,
        };

        if !self.expect(vec![Expected::Token(TokenKind::Semicolon)]) {
            return None;
        }

        Some(stmt.spanned(stmt_expr_span))
    }

    pub fn parse_variable_declaration(&mut self) -> Option<Statement> {
        let decl_tok = self.current_token();
        let decl_tok_kind = decl_tok.kind;
        let decl_span = decl_tok.span;
        self.advance();

        if !self.check(vec![Expected::Token(TokenKind::Identifier)]) {
            return None;
        }

        let identifier = self.current_token().literal.get_identifier()?;
        self.advance();

        // TODO: For now, types are EXPLICITLY DEFINED, change it to
        // accept implicitly typed identifier
        if !self.expect(vec![Expected::Token(TokenKind::Colon)]) {
            return None;
        }

        let value_type = self.try_parse_value_type()?;

        if !self.expect(vec![Expected::Token(TokenKind::Assignment)]) {
            return None;
        }

        let value = self.try_parse_expr(Precedence::Default)?;
        let value_span = value.span;

        if !self.expect(vec![Expected::Token(TokenKind::Semicolon)]) {
            return None;
        };

        let stmt = Stmt::VariableDeclaration {
            mutable: decl_tok_kind == TokenKind::Let,
            identifier,
            annotated_type: value_type,
            value,
        };
        let full_span = Span {
            ln_start: decl_span.ln_start,
            ln_end: value_span.ln_end,
            col_start: decl_span.col_start,
            col_end: value_span.col_end,
        };

        Some(stmt.spanned(full_span))
    }
}
