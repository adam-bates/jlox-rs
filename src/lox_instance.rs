use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    lox_callable::LoxCallable,
    lox_class::LoxClass,
    runtime_value::{RuntimeError, RuntimeResult, RuntimeValue},
    string::LoxStr,
    token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    pub class: LoxClass,
    pub fields: Rc<RefCell<HashMap<LoxStr, RuntimeValue>>>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        return Self {
            class,
            fields: Rc::new(RefCell::new(HashMap::new())),
        };
    }

    pub fn get(&self, name: &Token) -> RuntimeResult {
        if let Some(value) = self.fields.borrow().get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(method) = LoxClass::find_method(&self.class.methods.borrow(), &name.lexeme) {
            return Ok(RuntimeValue::LoxCallable(LoxCallable::LoxFunction(
                method.bind(self.clone()),
            )));
        };

        return Err(RuntimeError::UndefinedProperty {
            name: name.clone(),
            details: Some(format!("Undefined property '{}'", name.lexeme)),
        });
    }

    pub fn set(&mut self, name: Token, value: RuntimeValue) {
        self.fields.borrow_mut().insert(name.lexeme, value);
    }
}
