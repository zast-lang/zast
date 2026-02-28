use crate::{
    ast::{FunctionParameter, Statement, Stmt},
    error_handler::zast_errors::Expected,
    lexer::tokens::{Span, TokenKind},
    parser::{ZastParser, precedence_table::Precedence},
};

impl ZastParser {
    /// Attempts to parse a single statement.
    ///
    /// First checks the statement lookup table for the current token. If a
    /// registered statement handler exists, it is dispatched. Otherwise, the
    /// parser falls back to parsing an expression statement, which must be
    /// terminated by a `;`.
    ///
    /// # Returns
    ///
    /// `Some(Statement)` on success, or `None` if parsing fails.
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

    /// Parses a function declaration, e.g. `fn foo(a: i32): void { ... }`.
    ///
    /// Consumes the `fn` keyword, then parses the function name, parameter
    /// list, `:` separator, return type, and body block in order.
    pub fn parse_function_declaration(&mut self) -> Option<Statement> {
        let fn_tok_span = self.current_token().span;
        self.advance(); // eat 'fn'

        if !self.check(vec![Expected::Token(TokenKind::Identifier)]) {
            return None;
        }

        let fn_name = self.current_token().literal.get_identifier()?;
        self.advance();

        let parameters = self.parse_function_parameter()?;

        if !self.expect(vec![Expected::Token(TokenKind::Colon)]) {
            return None;
        }

        let return_type = self.try_parse_return_type()?;
        let body = self.parse_block_statement()?;
        let body_span = body.span;

        let full_span = Span {
            ln_start: fn_tok_span.ln_start,
            ln_end: body_span.ln_end,
            col_start: fn_tok_span.col_start,
            col_end: body_span.col_end,
        };

        Some(
            Stmt::FunctionDeclaration {
                name: fn_name,
                parameters,
                return_type,
                body: Box::new(body),
            }
            .spanned(full_span),
        )
    }

    /// Parses a function parameter list, e.g. `(a: i32, b: *u8,)`.
    ///
    /// Expects an opening `(`. Returns an empty vec for `()`. Supports
    /// optional trailing commas. Each parameter is a name-type pair separated
    /// by `:`.
    fn parse_function_parameter(&mut self) -> Option<Vec<FunctionParameter>> {
        if !self.expect(vec![Expected::Token(TokenKind::LeftParenthesis)]) {
            return None;
        }

        let mut params = Vec::new();

        // empty parameter list
        if self.current_token_kind() == TokenKind::RightParenthesis {
            self.advance();
            return Some(params);
        }

        params.push(self.parse_single_param()?);

        while !self.is_at_eof() && self.current_token_kind() == TokenKind::Comma {
            self.advance(); // eat ','

            // optional trailing comma
            if self.current_token_kind() == TokenKind::RightParenthesis {
                break;
            }

            params.push(self.parse_single_param()?);
        }

        if !self.expect(vec![Expected::Token(TokenKind::RightParenthesis)]) {
            return None;
        }

        Some(params)
    }

    /// Parses a single function parameter, e.g. `a: i32` or `ptr: *u8`.
    ///
    /// Expects the form: `<identifier> : <type>`
    ///
    /// Used by [`ZastParser::parse_function_parameter`] to avoid duplicating
    /// parameter parsing logic between the first parameter and subsequent ones.
    fn parse_single_param(&mut self) -> Option<FunctionParameter> {
        if !self.check(vec![Expected::Token(TokenKind::Identifier)]) {
            return None;
        }

        let name = self.current_token().literal.get_identifier()?;
        self.advance();

        if !self.expect(vec![Expected::Token(TokenKind::Colon)]) {
            return None;
        }

        let annotated_type = self.try_parse_value_type()?;

        Some(FunctionParameter {
            name,
            annotated_type,
        })
    }

    /// Parses a block statement, e.g. `{ let x: i32 = 5; }`.
    ///
    /// Consumes the opening `{`, parses zero or more statements until `}` is
    /// reached, then consumes the closing `}`. The full span covers from `{`
    /// to `}` inclusive.
    fn parse_block_statement(&mut self) -> Option<Statement> {
        let lb_span = self.current_token().span;

        if !self.expect(vec![Expected::Token(TokenKind::LeftBrace)]) {
            return None;
        }

        let mut stmts = Vec::new();

        while !self.is_at_eof() && self.current_token_kind() != TokenKind::RightBrace {
            let stmt = self.try_parse_stmt()?;
            stmts.push(Box::new(stmt));
        }

        let rb_span = self.current_token().span;

        if !self.expect(vec![Expected::Token(TokenKind::RightBrace)]) {
            return None;
        }

        let full_span = Span {
            ln_start: lb_span.ln_start,
            ln_end: rb_span.ln_end,
            col_start: lb_span.col_start,
            col_end: rb_span.col_end,
        };

        Some(Stmt::BlockStatement { statements: stmts }.spanned(full_span))
    }

    /// Parses a variable declaration, e.g. `const x: i32 = 5;` or `let y: *u8 = &x;`.
    ///
    /// The mutability of the variable is determined by the declaring keyword:
    /// `let` produces a mutable binding, `const` produces an immutable one.
    ///
    /// Expects the form: `<keyword> <identifier> : <type> = <expr> ;`
    pub fn parse_variable_declaration(&mut self) -> Option<Statement> {
        let decl_tok_kind = self.current_token().kind;
        let decl_span = self.current_token().span;
        self.advance(); // eat 'let' or 'const'

        if !self.check(vec![Expected::Token(TokenKind::Identifier)]) {
            return None;
        }

        let identifier = self.current_token().literal.get_identifier()?;
        self.advance();

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
        }

        let full_span = Span {
            ln_start: decl_span.ln_start,
            ln_end: value_span.ln_end,
            col_start: decl_span.col_start,
            col_end: value_span.col_end,
        };

        Some(
            Stmt::VariableDeclaration {
                mutable: decl_tok_kind == TokenKind::Let,
                identifier,
                annotated_type: value_type,
                value,
            }
            .spanned(full_span),
        )
    }
}
