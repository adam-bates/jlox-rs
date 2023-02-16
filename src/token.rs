use crate::{string::LoxStr, token_type::TokenType};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: LoxStr,
    pub line: usize,
}
