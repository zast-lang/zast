/// All token types recognized by the Zast lexer.
///
/// Variants are grouped by category:
///
/// ```text
/// Special      Illegal, Eof
/// Literals     String, Identifier, Integer, Float
/// Punctuation  Semicolon, Comma, Dot
/// Delimiters   LeftParenthesis, RightParenthesis
/// Operators    Plus, Minus, Multiply, Divide
/// Keywords     Let
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

    /// ':'
    Colon,

    /// '='
    Assignment,

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

    /// `let` keyword — introduces a mutable variable declaration.
    Let,

    /// `const` keyword — introduces a mutable variable declaration.
    Const,
}

/// The literal value carried by a token, tagged by its kind.
///
/// Only token kinds that have an associated runtime value produce a non-[`Literal::None`]
/// variant. All other tokens (operators, punctuation, keywords) use [`Literal::None`].
#[derive(Debug, Clone)]
pub enum Literal {
    /// A string literal value, e.g. the contents of `"hello"` excluding quotes.
    StringValue(String),

    /// A 64-bit signed integer value, e.g. `42`.
    IntegerValue(i64),

    /// A 64-bit floating-point value, e.g. `3.14`.
    FloatValue(f64),

    /// A user-defined identifier name, e.g. `foo`, `_bar`.
    Identifier(String),

    /// No literal value — used for operators, punctuation, and keywords.
    None,
}

impl Literal {
    /// Constructs a [`Literal`] from a [`TokenKind`] and its raw source string.
    ///
    /// Maps the token kind to the appropriate literal variant, parsing numeric
    /// strings into their respective types. Falls back to [`Literal::None`]
    /// for unrecognized kinds.
    ///
    /// # Arguments
    ///
    /// * `token_kind` - The kind of token being constructed.
    /// * `literal`    - The raw source string of the token.
    ///
    /// # Panics
    ///
    /// Panics if `token_kind` is [`TokenKind::Integer`] or [`TokenKind::Float`]
    /// and `literal` is not a valid number string.
    pub fn from(token_kind: &TokenKind, literal: String) -> Self {
        match token_kind {
            TokenKind::String => Literal::StringValue(literal),
            TokenKind::Identifier => Literal::Identifier(literal),
            TokenKind::Integer => Literal::IntegerValue(literal.parse().unwrap()),
            TokenKind::Float => Literal::FloatValue(literal.parse().unwrap()),
            _ => Literal::None,
        }
    }

    /// Returns the inner string value if this is a [`Literal::StringValue`], otherwise `None`.
    pub fn get_string(&self) -> Option<String> {
        match self {
            Self::StringValue(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Returns the inner integer value if this is a [`Literal::IntegerValue`], otherwise `None`.
    pub fn get_int(&self) -> Option<i64> {
        match self {
            Self::IntegerValue(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the inner float value if this is a [`Literal::FloatValue`], otherwise `None`.
    pub fn get_float(&self) -> Option<f64> {
        match self {
            Self::FloatValue(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the inner identifier string if this is a [`Literal::Identifier`], otherwise `None`.
    pub fn get_identifier(&self) -> Option<String> {
        match self {
            Self::Identifier(v) => Some(v.clone()),
            _ => None,
        }
    }
}

impl TokenKind {
    /// Returns `true` if this token kind carries a literal value.
    ///
    /// Literal tokens are those that have an associated [`Literal`] value
    /// rather than being purely structural (operators, keywords, punctuation).
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
    /// The classified literal value of this token, if any.
    /// Operators, keywords, and punctuation carry [`Literal::None`].
    pub literal: Literal,

    /// The raw source text of this token exactly as it appeared in the input.
    pub lexeme: String,

    /// The source location of this token.
    pub span: Span,

    /// The classification of this token.
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
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            _ => TokenKind::Identifier,
        };

        let literal = match token_kind {
            TokenKind::Identifier => Literal::Identifier(keyword.to_string()),
            _ => Literal::None,
        };

        Self {
            literal,
            lexeme: keyword.to_string(),
            span,
            kind: token_kind,
        }
    }
}

impl Default for Token {
    /// Returns an EOF token with an empty lexeme and a zeroed span.
    ///
    /// Useful as a safe sentinel value when the token stream is exhausted
    /// or when a token is needed before any real input has been scanned.
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
#[derive(Default, Debug, Clone, Copy)]
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
