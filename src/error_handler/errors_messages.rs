use crate::error_handler::zast_errors::ZastError;

impl ZastError {
    pub fn get_error_msg(&self) -> String {
        match self {
            Self::UnexpectedToken { token_kind, .. } => {
                format!("Unexpected token found '{:?}'", token_kind)
            }
            Self::ExpectedToken {
                expected_tokens,
                found_token,
                ..
            } => {
                if expected_tokens.len() == 1 {
                    format!(
                        "Expected {}, got '{:?}' instead",
                        expected_tokens[0], found_token
                    )
                } else {
                    let tok_str: Vec<String> =
                        expected_tokens.iter().map(|t| t.to_string()).collect();
                    let tokens = tok_str.join(", ");

                    format!(
                        "Expected either one of ( {} ), got '{:?}' instead",
                        tokens, found_token
                    )
                }
            }
            Self::IllegalToken { token_lexeme, .. } => {
                format!("Illegal token found '{}'", token_lexeme)
            }
        }
    }
}
