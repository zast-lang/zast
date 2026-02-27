/// All token types recognized by the Zast lexer.
///
/// Variants are grouped by category:
///
/// ```text
/// Special     Illegal, Eof
/// Literals    String, Identifier, Integer, Float
/// Punctuation Semicolon, Comma, Dot
/// Operators   Plus, Minus, Multiply, Divide
/// Keywords    Var
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TokenKind {
    /// A character or sequence that does not match any known lexeme.
    Illegal,

    /// Marks the end of the token stream.
    Eof,

    /// A string literal, e.g. `"hello"`. The inner value excludes the quotes.
    String,

    /// A user-defined name, e.g. `foo`, `_bar`, `myVar`.
    Identifier,

    /// A 64-bit signed integer literal, e.g. `42`.
    Integer,

    /// A 64-bit floating-point literal, e.g. `3.14`.
    Float,

    /// `;`
    Semicolon,

    /// `,`
    Comma,

    /// `.`
    Dot,

    /// `+`
    Plus,

    /// `-`
    Minus,

    /// `*`
    Multiply,

    /// `/`
    Divide,

    /// `(`
    LeftParenthesis,

    /// `)`
    RightParenthesis,

    /// `var` keyword — introduces a variable declaration.
    Var,
}

#[derive(Debug)]
pub enum Literal {
    StringValue(String),
    IntegerValue(i64),
    Identifier(String),
    FloatValue(f64),
    String(String),
    None,
}

impl Literal {
    pub fn from(token_kind: &TokenKind, literal: String) -> Self {
        match token_kind {
            TokenKind::String => Literal::StringValue(literal),
            TokenKind::Identifier => Literal::Identifier(literal),
            TokenKind::Integer => Literal::IntegerValue(literal.parse().unwrap()),
            TokenKind::Float => Literal::FloatValue(literal.parse().unwrap()),
            _ => Literal::String(literal),
        }
    }
}

impl TokenKind {
    pub fn is_delimiter(&self) -> bool {
        match self {
            Self::Semicolon => true,
            _ => false,
        }
    }

    pub fn is_literal_value(&self) -> bool {
        match self {
            Self::Identifier | Self::Integer | Self::String | Self::Float => true,
            _ => false,
        }
    }
}

/// A single lexeme produced by the lexer, carrying its kind, raw source text,
/// and the location in the source where it appeared.
#[derive(Debug)]
pub struct Token {
    /// The raw source text of the token, exactly as it appeared in the input.
    pub literal: Literal,

    pub lexeme: String,

    /// The source location of this token.
    pub span: Span,

    /// The classification and any associated value of this token.
    pub kind: TokenKind,
}

impl Token {
    /// Constructs a [`Token`] from a scanned identifier string.
    ///
    /// If `keyword` matches a reserved word, the appropriate [`TokenKind`] is
    /// assigned. Otherwise the token is classified as [`TokenKind::Identifier`].
    ///
    /// # Arguments
    ///
    /// * `keyword` - The raw scanned string to classify.
    /// * `span`    - The source location of the scanned string.
    pub fn from_keyword(keyword: &str, span: Span) -> Self {
        let token_kind = match keyword {
            "var" => TokenKind::Var,
            _ => TokenKind::Identifier,
        };

        Self {
            literal: Literal::None,
            lexeme: keyword.to_string(),
            span,
            kind: token_kind,
        }
    }
}

impl Default for Token {
    /// Returns an EOF token with an empty literal and a zeroed span.
    fn default() -> Self {
        Self {
            literal: Literal::None,
            lexeme: String::new(),
            span: Span::default(),
            kind: TokenKind::Eof,
        }
    }
}

/// A half-open source range `[ln_start:col_start, ln_end:col_end]` identifying
/// where a token appears in the original source text.
///
/// Both lines and columns are 1-based. A single-character token on line 3,
/// column 7 would have `ln_start = ln_end = 3` and `col_start = col_end = 7`.
#[derive(Default, Debug)]
pub struct Span {
    /// 1-based column of the first character of the token.
    pub col_start: usize,

    /// 1-based column of the last character of the token.
    pub col_end: usize,

    /// 1-based line number of the first character of the token.
    pub ln_start: usize,

    /// 1-based line number of the last character of the token.
    pub ln_end: usize,
}
