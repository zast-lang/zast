pub mod expressions;
pub mod precedence_table;
pub mod statements;
pub mod types;

use std::{collections::HashMap, mem};

use crate::{
    ast::{Expression, Statement, ZastProgram},
    error_handler::{
        ZastErrorCollector,
        zast_errors::{Expected, ZastError},
    },
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
        parser.register_nud(TokenKind::Float, ZastParser::parse_float_literal);
        parser.register_nud(
            TokenKind::LeftParenthesis,
            ZastParser::parse_grouping_expression,
        );

        parser.register_led(TokenKind::Plus, ZastParser::parse_binary_expr);
        parser.register_led(TokenKind::Minus, ZastParser::parse_binary_expr);
        parser.register_led(TokenKind::Divide, ZastParser::parse_binary_expr);
        parser.register_led(TokenKind::Multiply, ZastParser::parse_binary_expr);

        parser.register_stmt(TokenKind::Let, ZastParser::parse_variable_declaration);
        parser.register_stmt(TokenKind::Const, ZastParser::parse_variable_declaration);
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
            } else {
                body.push(node.unwrap());
            }
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

    pub(crate) fn throw_error(&mut self, err: ZastError) {
        self.errors.add_error(err);
    }

    fn sync_tokens(&mut self) {
        let mut depth = 0;

        while !self.is_at_eof() {
            match self.current_token_kind() {
                TokenKind::LeftParenthesis => {
                    depth += 1;
                    self.advance();
                }

                TokenKind::RightParenthesis => {
                    if depth == 0 {
                        self.advance();
                        return;
                    }
                    depth -= 1;
                    self.advance();
                }

                TokenKind::Semicolon => {
                    self.advance();
                    if depth == 0 {
                        return;
                    }
                }

                TokenKind::Eof => return,

                _ => {
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn current_token(&self) -> &Token {
        &self.tokens[self.current_token_ptr]
    }

    pub(crate) fn peek_token(&self) -> &Token {
        self.peek_at(1)
    }

    pub(crate) fn peek_at(&self, n: usize) -> &Token {
        if self.current_token_ptr + n >= self.tokens.len() {
            return &self.tokens[self.current_token_ptr];
        }
        &self.tokens[self.current_token_ptr + n]
    }

    pub(crate) fn peek_token_kind(&self) -> TokenKind {
        self.peek_token().kind
    }

    pub(crate) fn current_token_kind(&self) -> TokenKind {
        self.current_token().kind
    }

    pub(crate) fn current_token_precedence(&self) -> u8 {
        Precedence::get_precedence(self.current_token_kind())
            .map(|p| p.into())
            .unwrap_or(0)
    }

    pub(crate) fn advance(&mut self) {
        if self.current_token_ptr + 1 < self.tokens.len() {
            self.current_token_ptr += 1;
        }
    }

    fn is_at_eof(&self) -> bool {
        self.current_token_kind() == TokenKind::Eof
    }

    pub(crate) fn expect(&mut self, expected: Vec<Expected>) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn check(&mut self, expected: Vec<Expected>) -> bool {
        let tok = self.current_token();
        let tok_kind = self.current_token_kind();

        let matches = expected.iter().any(|e| match e {
            Expected::Token(kind) => tok_kind == *kind,
            Expected::Concept(_) => false,
        });

        if matches {
            return true;
        }

        self.errors.add_error(ZastError::ExpectedToken {
            span: tok.span,
            expected_tokens: expected,
            found_token: tok_kind,
        });

        false
    }
}
