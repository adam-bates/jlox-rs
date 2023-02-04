use std::collections::HashMap;

use crate::{
    runtime_value::{RuntimeError, RuntimeResult, RuntimeValue},
    string::LoxStr,
    token::Token,
};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<LoxStr, RuntimeValue>,
}

impl Environment {
    pub fn new() -> Self {
        return Self {
            values: HashMap::new(),
        };
    }

    pub fn define(&mut self, name: LoxStr, value: RuntimeValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<&RuntimeValue> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value);
        }

        return Err(RuntimeError::UndefinedVariable {
            name: name.clone(),
            details: None,
        });
    }
}
