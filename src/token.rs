use crate::{token_type::TokenType, string::LoxStr};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: LoxStr,
    pub line: usize,
}
