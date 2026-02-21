use crate::lexer::tokens::{Span, Token, TokenKind};
use std::mem;

/// A lexer for the Zast language.
///
/// `ZastLexer` transforms raw source text into a flat sequence of [`Token`]s,
/// annotated with precise source locations ([`Span`]s). It operates as a
/// single-pass, character-by-character scanner with one character of lookahead.
///
/// # Errors
///
/// If the lexer encounters characters it cannot classify, it emits
/// [`TokenKind::Illegal`] tokens and continues. Accumulated errors are
/// returned as a `Vec<String>` from [`ZastLexer::tokenize`].
///
/// # Example
///
/// ```rust
/// let mut lexer = ZastLexer::new("10 + 3.14");
/// match lexer.tokenize() {
///     Ok(tokens) => lexer.debug_tokens(tokens),
///     Err(errors) => errors.iter().for_each(|e| eprintln!("Error: {e}")),
/// }
/// ```
pub struct ZastLexer {
    /// The source text, stored as a character vector for O(1) indexed access.
    source: Vec<char>,

    /// Accumulated lexical errors encountered during tokenization.
    errors: Vec<String>,

    /// The token stream produced so far.
    tokens: Vec<Token>,

    /// Absolute byte position of the current character within `source`.
    current_source_pos: usize,

    /// The 1-based line number of the current character.
    current_line: usize,

    /// The 1-based column number of the current character within its line.
    current_column: usize,
}

impl ZastLexer {
    /// Creates a new `ZastLexer` from a source string.
    ///
    /// The lexer begins at line 1, column 1. No tokenization is performed
    /// until [`ZastLexer::tokenize`] is called.
    ///
    /// # Arguments
    ///
    /// * `src` - The full source text to lex.
    pub fn new(src: &str) -> Self {
        Self {
            source: src.chars().collect(),
            errors: Vec::new(),
            tokens: Vec::new(),
            current_source_pos: 0,
            current_line: 1,
            current_column: 1,
        }
    }

    /// Prints a human-readable debug representation of a token sequence.
    ///
    /// Outputs each token's literal value, kind, and source span in the format:
    /// `ln_start:col_start–ln_end:col_end`.
    ///
    /// # Arguments
    ///
    /// * `toks` - The token sequence to display, typically from [`ZastLexer::tokenize`].
    pub fn debug_tokens(&self, toks: Vec<Token>) {
        for t in toks {
            println!("TokenLiteral: {:?}", t.literal);
            println!("TokenKind: {:?}", t.kind);
            println!(
                "TokenSpan: {}:{}–{}:{}",
                t.span.ln_start, t.span.col_start, t.span.ln_end, t.span.col_end
            );
            println!()
        }
    }

    /// Tokenizes the entire source text.
    ///
    /// Drives the lexer from the start of the source to the end, producing
    /// one [`Token`] per lexeme. Whitespace and newlines are skipped between
    /// tokens. Line and column counters are updated as the source is consumed.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Token>)` if no errors were encountered.
    /// - `Err(Vec<String>)` containing all accumulated error messages if any
    ///   illegal characters were encountered.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, Vec<String>> {
        while !self.is_at_end() {
            self.skip_whitespaces();

            let token = self.read_token();
            self.tokens.push(token);

            self.skip_whitespaces();
        }
        self.tokens
            .push(self.new_token(TokenKind::Eof, String::from("END_OF_FILE")));

        if self.errors.is_empty() {
            Ok(mem::take(&mut self.tokens))
        } else {
            Err(mem::take(&mut self.errors))
        }
    }

    /// Dispatches to the appropriate sub-lexer based on the current character.
    ///
    /// Numeric characters are routed to [`ZastLexer::tokenize_number`],
    /// alphabetic characters and underscores to [`ZastLexer::tokenize_keyword`],
    /// and all known punctuation and operators are matched directly. Unrecognized
    /// characters produce an [`TokenKind::Illegal`] token.
    fn read_token(&mut self) -> Token {
        let cur = self.current_char();
        let strc = String::from(cur);

        if self.is_number(cur) {
            return self.tokenize_number();
        }

        if self.is_alphabetic(cur) {
            return self.tokenize_keyword();
        }

        let tok = match cur {
            ';' => self.new_token(TokenKind::Semicolon, strc),
            '.' => self.new_token(TokenKind::Dot, strc),
            '+' => self.new_token(TokenKind::Plus, strc),
            '-' => self.new_token(TokenKind::Minus, strc),
            '*' => self.new_token(TokenKind::Multiply, strc),
            '/' => self.new_token(TokenKind::Divide, strc),
            _ => self.new_token(TokenKind::Illegal, strc),
        };

        self.advance();

        tok
    }

    /// Scans a keyword or identifier starting at the current position.
    ///
    /// Consumes all alphanumeric characters and underscores following the
    /// initial alphabetic character. The resulting string is matched against
    /// the set of reserved keywords via [`Token::from_keyword`]; if no keyword
    /// matches, an identifier token is produced instead.
    fn tokenize_keyword(&mut self) -> Token {
        let col_start = self.current_column;
        let ln_start = self.current_line;
        let src_start = self.current_source_pos;

        while !self.is_at_end() && self.is_alphanumeric(self.peek_char()) {
            self.advance();
        }

        self.advance();

        let col_end = self.current_column - 1;
        let ln_end = self.current_line;
        let src_end = self.current_source_pos;

        let keyword_src: String = self.source[src_start..src_end].iter().collect();

        Token::from_keyword(
            &keyword_src,
            self.get_span(col_start, col_end, ln_start, ln_end),
        )
    }

    /// Scans an integer or floating-point numeric literal.
    ///
    /// Consumes an uninterrupted sequence of ASCII digits. If the sequence is
    /// immediately followed by a `.` and at least one more digit, the lexer
    /// continues scanning and produces a [`TokenKind::Float`]. Otherwise it
    /// produces a [`TokenKind::Integer`].
    ///
    /// # Panics
    ///
    /// Panics if the scanned slice cannot be parsed as `i32` or `f64`. This
    /// should not occur under normal operation since only digit characters are
    /// consumed.
    fn tokenize_number(&mut self) -> Token {
        let col_start = self.current_column;
        let ln_start = self.current_line;
        let src_start = self.current_source_pos;

        while !self.is_at_end() && self.is_number(self.peek_char()) {
            self.advance();
        }

        self.advance();

        if self.current_char_is('.') && self.is_number(self.peek_char()) {
            self.advance(); // consume '.'

            while self.is_number(self.peek_char()) {
                self.advance();
            }

            self.advance();

            let col_end = self.current_column - 1;
            let ln_end = self.current_line;
            let src_end = self.current_source_pos;

            let num_lit: String = self.source[src_start..src_end].iter().collect();
            let num = num_lit.parse::<f64>().unwrap();

            Token {
                literal: num_lit.clone(),
                kind: TokenKind::Float(num),
                span: self.get_span(col_start, col_end, ln_start, ln_end),
            }
        } else {
            let col_end = self.current_column - 1;
            let ln_end = self.current_line;
            let src_end = self.current_source_pos;

            let num_lit: String = self.source[src_start..src_end].iter().collect();
            let num = num_lit.parse::<i32>().unwrap();

            Token {
                literal: num_lit.clone(),
                kind: TokenKind::Integer(num),
                span: self.get_span(col_start, col_end, ln_start, ln_end),
            }
        }
    }

    /// Advances past any whitespace characters, updating line and column state.
    ///
    /// Newlines (`\n`) increment the line counter and reset the column to 1.
    /// Carriage returns, tabs, and spaces are consumed silently.
    fn skip_whitespaces(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                '\n' => {
                    self.advance();
                    self.current_line += 1;
                    self.current_column = 1;
                }
                '\r' | '\t' | ' ' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// Returns the character at the current position without advancing.
    fn current_char(&self) -> char {
        self.source[self.current_source_pos]
    }

    /// Returns `true` if the character at the current position equals `c`.
    fn current_char_is(&self, c: char) -> bool {
        self.current_char() == c
    }

    /// Returns the character immediately following the current position without advancing.
    ///
    /// Returns `'\0'` if the current position is the last character in the source,
    /// i.e. there is no next character to peek at.
    fn peek_char(&self) -> char {
        if self.current_source_pos + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current_source_pos + 1]
    }

    /// Advances the lexer by one character, updating source position and column.
    ///
    /// Has no effect if the lexer is already at the end of the source.
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current_source_pos += 1;
            self.current_column += 1;
        }
    }

    /// Returns `true` if the lexer has consumed all characters in the source.
    fn is_at_end(&self) -> bool {
        self.current_source_pos >= self.source.len()
    }

    /// Returns `true` if `c` is an ASCII decimal digit.
    fn is_number(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    /// Returns `true` if `c` can begin an identifier: an alphabetic character or `_`.
    fn is_alphabetic(&self, c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    /// Returns `true` if `c` is valid inside an identifier: alphabetic, digit, or `_`.
    fn is_alphanumeric(&self, c: char) -> bool {
        self.is_alphabetic(c) || self.is_number(c)
    }

    /// Constructs a [`Span`] from explicit line and column bounds.
    fn get_span(&self, col_start: usize, col_end: usize, ln_start: usize, ln_end: usize) -> Span {
        Span {
            col_start,
            col_end,
            ln_start,
            ln_end,
        }
    }

    /// Constructs a single-character [`Token`] at the current source position.
    ///
    /// The span covers exactly the current column on the current line.
    fn new_token(&self, token_kind: TokenKind, literal: String) -> Token {
        let span = self.get_span(
            self.current_column,
            self.current_column,
            self.current_line,
            self.current_line,
        );

        Token {
            literal,
            kind: token_kind,
            span,
        }
    }
}
