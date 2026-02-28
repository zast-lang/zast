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

/// A function that parses a null-denotation (prefix) expression.
///
/// NUD functions are called when a token appears at the start of an expression
/// (i.e. in prefix position). Examples include integer literals, identifiers,
/// unary operators, and grouping parentheses.
type NUDParseFn = fn(&mut ZastParser) -> Option<Expression>;

/// A function that parses a left-denotation (infix) expression.
///
/// LED functions are called when a token appears after a left-hand expression
/// (i.e. in infix position). Examples include binary operators like `+`, `-`,
/// `*`, and `/`.
type LEDParseFn = fn(&mut ZastParser, Expression) -> Option<Expression>;

/// A function that parses a statement.
///
/// Statement functions are dispatched when a token appears at the start of a
/// statement context. Examples include `let`, `const`, and `fn`.
type StmtParseFn = fn(&mut ZastParser) -> Option<Statement>;

/// A Pratt parser for the Zast language.
///
/// `ZastParser` transforms a flat [`Token`] stream (produced by [`ZastLexer`])
/// into a structured [`ZastProgram`] AST. It uses a Pratt (top-down operator
/// precedence) parsing strategy, dispatching to NUD, LED, and statement handler
/// functions via lookup tables keyed on [`TokenKind`].
///
/// # Error Recovery
///
/// When a parse function fails, the parser calls [`ZastParser::sync_tokens`]
/// to resynchronize at the next statement boundary (`;`, `}`, or EOF),
/// allowing parsing to continue and accumulate further errors.
///
/// # Example
///
/// ```rust
/// let mut parser = ZastParser::new(tokens);
/// match parser.parse_program() {
///     Ok(program) => println!("{:#?}", program),
///     Err(errors) => errors.iter().for_each(|e| eprintln!("Error: {e}")),
/// }
/// ```
pub struct ZastParser {
    /// The flat token stream produced by the lexer.
    tokens: Vec<Token>,

    /// Index of the current token being examined.
    current_token_ptr: usize,

    /// Accumulated parse errors encountered during parsing.
    errors: ZastErrorCollector,

    /// Lookup table mapping token kinds to NUD (prefix) parse functions.
    nud_lookup: HashMap<TokenKind, NUDParseFn>,

    /// Lookup table mapping token kinds to LED (infix) parse functions.
    led_lookup: HashMap<TokenKind, LEDParseFn>,

    /// Lookup table mapping token kinds to statement parse functions.
    stmt_lookup: HashMap<TokenKind, StmtParseFn>,
}

impl ZastParser {
    /// Creates a new `ZastParser` from a token stream and registers all
    /// built-in NUD, LED, and statement parse functions.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The token stream produced by [`ZastLexer::tokenize`].
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Self {
            tokens,
            current_token_ptr: 0,
            errors: ZastErrorCollector::new(),
            nud_lookup: HashMap::new(),
            led_lookup: HashMap::new(),
            stmt_lookup: HashMap::new(),
        };

        parser.register_nud(TokenKind::Multiply, ZastParser::parse_deref_expr);
        parser.register_nud(TokenKind::Ampersand, ZastParser::parse_addr_expr);
        parser.register_nud(TokenKind::Integer, ZastParser::parse_integer_literal);
        parser.register_nud(TokenKind::Float, ZastParser::parse_float_literal);
        parser.register_nud(TokenKind::Identifier, ZastParser::parse_identifier_literal);
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
        parser.register_stmt(TokenKind::Fn, ZastParser::parse_function_declaration);

        parser
    }

    /// Registers a NUD (prefix) parse function for the given token kind.
    pub fn register_nud(&mut self, token_kind: TokenKind, nud_fn: NUDParseFn) {
        self.nud_lookup.insert(token_kind, nud_fn);
    }

    /// Registers a LED (infix) parse function for the given token kind.
    pub fn register_led(&mut self, token_kind: TokenKind, led_fn: LEDParseFn) {
        self.led_lookup.insert(token_kind, led_fn);
    }

    /// Registers a statement parse function for the given token kind.
    pub fn register_stmt(&mut self, token_kind: TokenKind, stmt_fn: StmtParseFn) {
        self.stmt_lookup.insert(token_kind, stmt_fn);
    }

    /// Parses the entire token stream into a [`ZastProgram`].
    ///
    /// Iterates over all tokens, attempting to parse each top-level statement.
    /// If a statement parse fails, [`ZastParser::sync_tokens`] is called to
    /// recover and continue parsing remaining statements.
    ///
    /// # Returns
    ///
    /// - `Ok(ZastProgram)` if no errors were encountered.
    /// - `Err(ZastErrorCollector)` containing all accumulated errors otherwise.
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

    /// Wraps the parsed statement list into a [`ZastProgram`].
    fn finish(&self, body: Vec<Statement>) -> ZastProgram {
        ZastProgram { body }
    }

    /// Adds an error to the error collector.
    pub(crate) fn throw_error(&mut self, err: ZastError) {
        self.errors.add_error(err);
    }

    /// Resynchronizes the parser after a parse error.
    ///
    /// Advances tokens until a safe recovery point is found, respecting
    /// nesting depth so that delimiters inside blocks or parentheses are
    /// not mistakenly treated as recovery points.
    ///
    /// Recovery points are:
    /// - `;` at depth 0 — end of a statement
    /// - `)` or `}` at depth 0 — end of a block or parameter list
    /// - [`TokenKind::Eof`] — end of input
    fn sync_tokens(&mut self) {
        let mut depth = 0;

        while !self.is_at_eof() {
            match self.current_token_kind() {
                TokenKind::LeftParenthesis | TokenKind::LeftBrace => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::RightParenthesis | TokenKind::RightBrace => {
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

    /// Returns a reference to the current token without advancing.
    pub(crate) fn current_token(&self) -> &Token {
        &self.tokens[self.current_token_ptr]
    }

    /// Returns a reference to the token immediately following the current token.
    ///
    /// Equivalent to `peek_at(1)`.
    pub(crate) fn peek_token(&self) -> &Token {
        self.peek_at(1)
    }

    /// Returns a reference to the token `n` positions ahead of the current token.
    ///
    /// If the lookahead would exceed the token stream bounds, returns the
    /// current token as a safe sentinel.
    pub(crate) fn peek_at(&self, n: usize) -> &Token {
        if self.current_token_ptr + n >= self.tokens.len() {
            return &self.tokens[self.current_token_ptr];
        }
        &self.tokens[self.current_token_ptr + n]
    }

    /// Returns the [`TokenKind`] of the token immediately following the current token.
    pub(crate) fn peek_token_kind(&self) -> TokenKind {
        self.peek_token().kind
    }

    /// Returns the [`TokenKind`] of the current token.
    pub(crate) fn current_token_kind(&self) -> TokenKind {
        self.current_token().kind
    }

    /// Returns the precedence of the current token as a raw `u8`.
    ///
    /// Returns `0` if the current token has no registered precedence,
    /// effectively treating it as a non-operator.
    pub(crate) fn current_token_precedence(&self) -> u8 {
        Precedence::get_precedence(self.current_token_kind())
            .map(|p| p.into())
            .unwrap_or(0)
    }

    /// Advances the parser to the next token.
    ///
    /// Has no effect if the parser is already at the last token in the stream.
    pub(crate) fn advance(&mut self) {
        if self.current_token_ptr + 1 < self.tokens.len() {
            self.current_token_ptr += 1;
        }
    }

    /// Returns `true` if the current token is [`TokenKind::Eof`].
    fn is_at_eof(&self) -> bool {
        self.current_token_kind() == TokenKind::Eof
    }

    /// Checks whether the current token matches any of the expected tokens,
    /// then advances past it if so.
    ///
    /// If the current token does not match, an error is emitted and `false`
    /// is returned without advancing.
    ///
    /// # Arguments
    ///
    /// * `expected` - A list of acceptable [`Expected`] tokens or concepts.
    pub(crate) fn expect(&mut self, expected: Vec<Expected>) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Checks whether the current token matches any of the expected tokens
    /// without consuming it.
    ///
    /// Emits an [`ZastError::ExpectedToken`] error if no match is found.
    /// Unlike [`ZastParser::expect`], this method never advances the token pointer.
    ///
    /// # Arguments
    ///
    /// * `expected` - A list of acceptable [`Expected`] tokens or concepts.
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
