use crate::{expr::*, string::LoxStr, token_type::TokenType};

use thiserror::Error;

pub type RuntimeResult<T = RuntimeValue, E = RuntimeError> = Result<T, E>;

#[derive(Debug, PartialEq)]
pub enum RuntimeValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(LoxStr),
    Object(Box<RuntimeValue>),
}

impl From<&LiteralExpr> for RuntimeValue {
    fn from(value: &LiteralExpr) -> Self {
        return match (&value.0, &value.1.token_type) {
            (LiteralExprType::Nil, _) => Self::Nil,
            (LiteralExprType::True, _) => Self::Boolean(true),
            (LiteralExprType::False, _) => Self::Boolean(false),
            (LiteralExprType::String, TokenType::String(value)) => Self::String(value.clone()),
            (LiteralExprType::Number, TokenType::Number(value)) => Self::Number(*value),

            (literal, token) => panic!(
                "[{}:{}] Unexpected token for literal {literal:?}: {token:#?}",
                file!(),
                line!()
            ),
        };
    }
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("invalid unary expression: {expr:#?}. Details = {details:?}")]
    InvalidUnaryExpr {
        expr: UnaryExpr,
        details: Option<String>,
    },

    #[error("invalid binary expression: {expr:#?}. Details = {details:?}")]
    InvalidBinaryExpr {
        expr: BinaryExpr,
        details: Option<String>,
    },
}