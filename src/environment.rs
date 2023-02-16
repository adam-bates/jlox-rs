use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    runtime_value::{RuntimeError, RuntimeResult, RuntimeValue},
    string::LoxStr,
    token::Token,
};

#[derive(Debug, PartialEq)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<LoxStr, RuntimeValue>,
}

impl Environment {
    pub fn new() -> Self {
        return Self {
            enclosing: None,
            values: HashMap::new(),
        };
    }

    pub fn enclosed(enclosing: Rc<RefCell<Environment>>) -> Self {
        return Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        };
    }

    pub fn define(&mut self, name: LoxStr, value: RuntimeValue) {
        self.values.insert(name, value);
    }

    pub fn get_at(this: Rc<RefCell<Self>>, distance: usize, name: &Token) -> RuntimeResult {
        return Self::ancestor(this, distance)
            .borrow()
            .values
            .get(&name.lexeme)
            .cloned()
            .ok_or_else(|| RuntimeError::UndefinedVariable {
                name: name.clone(),
                details: None,
            });
    }

    pub fn get(&self, name: &Token) -> RuntimeResult {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }

        return Err(RuntimeError::UndefinedVariable {
            name: name.clone(),
            details: None,
        });
    }

    pub fn assign_at(
        this: Rc<RefCell<Self>>,
        distance: usize,
        name: Token,
        value: RuntimeValue,
    ) -> RuntimeResult<()> {
        let this = Self::ancestor(this, distance);

        if this.borrow().values.contains_key(&name.lexeme) {
            this.borrow_mut().values.insert(name.lexeme, value);
            return Ok(());
        }

        return Err(RuntimeError::UndefinedVariable {
            name: name.clone(),
            details: Some(format!("Cannot assign [{value:?}] to undefined variable")),
        });
    }

    pub fn assign(&mut self, name: Token, value: RuntimeValue) -> RuntimeResult<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        return Err(RuntimeError::UndefinedVariable {
            name: name.clone(),
            details: Some(format!("Cannot assign [{value:?}] to undefined variable")),
        });
    }

    fn ancestor(this: Rc<RefCell<Self>>, distance: usize) -> Rc<RefCell<Self>> {
        let mut environment = this;

        for _ in 0..distance {
            let cloned = Rc::clone(
                environment
                    .borrow()
                    .enclosing
                    .as_ref()
                    .expect("Resolver shouldn't pass invalid distance"),
            );

            environment = cloned;
        }

        return environment;
    }
}
