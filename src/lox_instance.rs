use std::collections::HashMap;

use crate::{
    lox_class::LoxClass,
    runtime_value::{RuntimeError, RuntimeResult, RuntimeValue},
    string::LoxStr,
    token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    pub class: LoxClass,
    pub fields: HashMap<LoxStr, RuntimeValue>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        return Self {
            class,
            fields: HashMap::new(),
        };
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<&RuntimeValue> {
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value);
        }

        return Err(RuntimeError::UndefinedProperty {
            name: name.clone(),
            details: Some(format!("Undefined property '{}'", name.lexeme)),
        });
    }
}
