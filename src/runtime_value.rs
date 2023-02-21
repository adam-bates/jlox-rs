use crate::{
    ast::expr::*, lox_callable::LoxCallable, lox_instance::LoxInstance, string::LoxStr,
    token::Token, token_type::TokenType,
};

use thiserror::Error;

pub type RuntimeResult<T = RuntimeValue, E = RuntimeError> = Result<T, E>;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(LoxStr),
    LoxCallable(LoxCallable),
    LoxInstance(LoxInstance),
}

impl From<&LiteralExpr> for RuntimeValue {
    fn from(value: &LiteralExpr) -> Self {
        return match (&value.literal_type, &value.token.token_type) {
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

    #[error("invalid get expression: {name:#?}. Details = {details:?}")]
    InvalidGetExpr {
        name: Token,
        details: Option<String>,
    },

    #[error("invalid set expression: {name:#?}. Details = {details:?}")]
    InvalidSetExpr {
        name: Token,
        details: Option<String>,
    },

    #[error("undefined variable: {name:#?}. Details = {details:?}")]
    UndefinedVariable {
        name: Token,
        details: Option<String>,
    },

    #[error("undefined property: {name:#?}. Details = {details:?}")]
    UndefinedProperty {
        name: Token,
        details: Option<String>,
    },

    #[error("invalid callable: {value:#?}. Details = {details:?}")]
    InvalidCallable {
        value: RuntimeValue,
        details: Option<String>,
    },

    #[error("function expected {expected} args, but call found {found}. Details = {details:?}")]
    WrongNumberOfArgs {
        expected: usize,
        found: usize,
        details: Option<String>,
    },

    #[error("non-error return short-circuit")]
    NonErrorReturnShortCircuit { value: Option<RuntimeValue> },
}
