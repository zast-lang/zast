use crate::{
    ast::Expression,
    error_handler::zast_errors::{Expected, ZastError},
    lexer::tokens::TokenKind,
    parser::ZastParser,
};

impl ZastParser {
    pub(crate) fn try_parse_value_type(&mut self) -> Option<Expression> {
        // TODO: Implement parser for compound types
        if self.current_token_kind() != TokenKind::Identifier {
            let cur_tok = self.current_token();
            self.throw_error(ZastError::ExpectedToken {
                span: cur_tok.span,
                expected_tokens: vec![Expected::Concept("type annotation")],
                found_token: cur_tok.kind,
            });
            return None;
        }
        self.parse_identifier_literal()
    }
}
