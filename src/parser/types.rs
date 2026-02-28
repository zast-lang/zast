use crate::{
    error_handler::zast_errors::{Expected, ZastError},
    lexer::tokens::TokenKind,
    parser::ZastParser,
    types::{annotated_type::AnnotatedType, return_type::ReturnType},
};

impl ZastParser {
    /// Parses a function return type annotation.
    ///
    /// If the current token is the identifier `"void"`, returns [`ReturnType::Void`].
    /// Otherwise delegates to [`ZastParser::try_parse_value_type`] and wraps
    /// the result in [`ReturnType::Type`].
    ///
    /// Note: `void` is treated as a reserved identifier rather than a keyword,
    /// consistent with Zast's design of keeping all type names as plain identifiers.
    pub(crate) fn try_parse_return_type(&mut self) -> Option<ReturnType> {
        let return_type_str = self.current_token().literal.get_identifier()?;

        if return_type_str == "void" {
            self.advance();
            return Some(ReturnType::Void);
        }

        let return_type = self.try_parse_value_type()?;
        Some(ReturnType::Type(return_type))
    }

    /// Parses a value type annotation, e.g. `i32`, `*u8`, `**bool`.
    ///
    /// Dispatches based on the current token:
    /// - `*` → pointer type via [`ZastParser::parse_pointer_type`]
    /// - `Identifier` → primitive type via [`ZastParser::parse_primitive_type`]
    /// - anything else → emits an error and returns `None`
    pub(crate) fn try_parse_value_type(&mut self) -> Option<AnnotatedType> {
        match self.current_token_kind() {
            TokenKind::Multiply => self.parse_pointer_type(),
            TokenKind::Identifier => self.parse_primitive_type(),
            _ => {
                let cur_tok = self.current_token();
                self.throw_error(ZastError::ExpectedToken {
                    span: cur_tok.span,
                    expected_tokens: vec![Expected::Concept("type annotation")],
                    found_token: cur_tok.kind,
                });
                None
            }
        }
    }

    /// Parses a pointer type annotation, e.g. `*u8`, `**i32`.
    ///
    /// Consumes the `*` token and recursively parses the pointee type via
    /// [`ZastParser::try_parse_value_type`], allowing arbitrary pointer depth.
    pub(crate) fn parse_pointer_type(&mut self) -> Option<AnnotatedType> {
        self.advance(); // eat '*'
        let inner = self.try_parse_value_type()?;
        Some(AnnotatedType::Pointer(Box::new(inner)))
    }

    /// Parses a primitive type annotation, e.g. `i32`, `u8`, `bool`.
    ///
    /// Consumes the identifier token and stores its name as a [`AnnotatedType::Primitive`].
    /// The name is resolved to a concrete type later during semantic analysis.
    pub(crate) fn parse_primitive_type(&mut self) -> Option<AnnotatedType> {
        let primitive = self.current_token().literal.get_identifier()?;
        self.advance();
        Some(AnnotatedType::Primitive(primitive))
    }
}
