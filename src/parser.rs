pub mod precedence_table;

use std::{collections::HashMap, mem};

use crate::{
    ast::{Expression, Statement, ZastProgram},
    lexer::tokens::{Token, TokenKind},
    parser::precedence_table::Precedence,
};

type NUDParseFn = fn(&mut ZastParser) -> Option<Expression>;
type LEDParseFn = fn(&mut ZastParser, Expression) -> Option<Expression>;
type StmtParseFn = fn(&mut ZastParser) -> Option<Statement>;

pub struct ZastParser {
    tokens: Vec<Token>,
    current_token_ptr: usize,
    errors: Vec<String>,

    nud_lookup: HashMap<TokenKind, NUDParseFn>,
    led_lookup: HashMap<TokenKind, LEDParseFn>,
    stmt_lookup: HashMap<TokenKind, StmtParseFn>,
}

impl ZastParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let parser = Self {
            tokens,
            current_token_ptr: 0,
            errors: Vec::new(),

            nud_lookup: HashMap::new(),
            led_lookup: HashMap::new(),
            stmt_lookup: HashMap::new(),
        };

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

    pub fn parse_program(&mut self) -> Result<ZastProgram, Vec<String>> {
        let mut body = Vec::new();
        while !self.is_at_eof() {
            let node = self.try_parse_stmt();

            if node.is_none() {
                self.sync_tokens();
            }

            body.push(node.unwrap());
        }

        if self.errors.is_empty() {
            Ok(self.finish(body))
        } else {
            Err(mem::take(&mut self.errors))
        }
    }

    fn finish(&self, body: Vec<Statement>) -> ZastProgram {
        ZastProgram { body }
    }

    fn throw_error(&mut self, err_msg: String) {
        self.errors.push(err_msg);
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

    fn advance(&mut self) {
        self.current_token_ptr += 1;
    }

    fn next_token_precedence(&self) -> u8 {
        Precedence::get_precedence(self.peek_token_kind()).into()
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
        // let stmt_expr = self.try_parse_expr()?;
        None
    }
    // ---------- Expressions ----------
    pub fn try_parse_expr(&mut self, precedence: u8) -> Option<Expression> {
        if let Some(left_fn) = self.nud_lookup.get(&self.current_token_kind()) {
            let mut left = left_fn(self)?;

            while precedence < self.next_token_precedence() && !self.is_at_eof() {
                if let Some(right_fn) = self.led_lookup.get(&self.current_token_kind()) {
                    left = right_fn(self, left_fn);
                }
                break;
            }

            return Some(left);
        }

        self.throw_error(format!(
            "Unexpected '{:?}' token",
            self.current_token_kind()
        ));
        return None;
    }
}
