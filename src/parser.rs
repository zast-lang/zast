pub mod precedence_table;

use std::{collections::HashMap, mem};

use crate::{
    ast::{Expr, Expression, Statement, Stmt, ZastProgram},
    error_handler::{ZastErrorCollector, zast_errors::ZastError},
    lexer::tokens::{Token, TokenKind},
    parser::precedence_table::Precedence,
};

type NUDParseFn = fn(&mut ZastParser) -> Option<Expression>;
type LEDParseFn = fn(&mut ZastParser, Expression) -> Option<Expression>;
type StmtParseFn = fn(&mut ZastParser) -> Option<Statement>;

pub struct ZastParser {
    tokens: Vec<Token>,
    current_token_ptr: usize,
    errors: ZastErrorCollector,

    nud_lookup: HashMap<TokenKind, NUDParseFn>,
    led_lookup: HashMap<TokenKind, LEDParseFn>,
    stmt_lookup: HashMap<TokenKind, StmtParseFn>,
}

impl ZastParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Self {
            tokens,
            current_token_ptr: 0,
            errors: ZastErrorCollector::new(),

            nud_lookup: HashMap::new(),
            led_lookup: HashMap::new(),
            stmt_lookup: HashMap::new(),
        };

        parser.register_nud(TokenKind::Integer, ZastParser::parse_integer_literal);
        parser
    }

    pub fn register_nud(&mut self, token_kind: TokenKind, nud_fn: NUDParseFn) {
        self.nud_lookup.insert(token_kind, nud_fn);
    }
    pub fn register_led(&mut self, token_kind: TokenKind, led_fn: LEDParseFn) {
        self.led_lookup.insert(token_kind, led_fn);
    }
    pub fn register_stmt(&mut self, token_kind: TokenKind, stmt_fn: StmtParseFn) {
        self.stmt_lookup.insert(token_kind, stmt_fn);
    }

    pub fn parse_program(&mut self) -> Result<ZastProgram, ZastErrorCollector> {
        let mut body = Vec::new();
        while !self.is_at_eof() {
            let node = self.try_parse_stmt();

            if node.is_none() {
                self.sync_tokens();
            }

            body.push(node.unwrap());
        }

        if self.errors.has_errors() {
            Err(mem::take(&mut self.errors))
        } else {
            Ok(self.finish(body))
        }
    }

    fn finish(&self, body: Vec<Statement>) -> ZastProgram {
        ZastProgram { body }
    }

    fn throw_error(&mut self, err: ZastError) {
        self.errors.add_error(err);
    }

    fn sync_tokens(&mut self) {
        while self.current_token_kind().is_delimiter() {
            self.advance();
        }

        self.advance(); // Advance past the delimiter
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.current_token_ptr]
    }

    fn peek_token(&self) -> &Token {
        &self.tokens[self.current_token_ptr + 1]
    }

    fn peek_token_kind(&self) -> TokenKind {
        self.peek_token().kind
    }

    fn current_token_kind(&self) -> TokenKind {
        self.current_token().kind
    }

    fn current_token_precedence(&self) -> u8 {
        Precedence::get_precedence(self.current_token_kind()).into()
    }

    fn next_token_precedence(&self) -> u8 {
        Precedence::get_precedence(self.peek_token_kind()).into()
    }

    fn advance(&mut self) {
        self.current_token_ptr += 1;
    }

    fn is_at_eof(&self) -> bool {
        self.current_token_kind() == TokenKind::Eof
    }

    fn expect(&mut self, expected_kind: TokenKind) -> bool {
        let tok_kind = self.current_token_kind();
        if tok_kind == expected_kind {
            self.advance();
            return true;
        }

        false
    }
    // ---------- Statements ----------
    pub fn try_parse_stmt(&mut self) -> Option<Statement> {
        if let Some(stmt_fn) = self.stmt_lookup.get(&self.current_token_kind()) {
            return stmt_fn(self);
        }

        let stmt_expr = self.try_parse_expr(Precedence::Default)?;
        let stmt_expr_span = stmt_expr.span;
        let stmt = Stmt::Expression {
            expression: stmt_expr,
        };

        Some(stmt.spanned(stmt_expr_span))
    }
    // ---------- Expressions ----------
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

                let led_fn = self.led_lookup.get(&self.current_token_kind());
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
}
