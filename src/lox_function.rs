use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    interpreter::Interpreter,
    runtime_value::{RuntimeResult, RuntimeValue},
    stmt::FunctionStmt,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    pub declaration: FunctionStmt,
}

impl LoxFunction {
    pub fn new(declaration: FunctionStmt) -> Self {
        return Self { declaration };
    }

    pub fn arity(&self) -> usize {
        return self.declaration.params.len();
    }

    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        mut arguments: Vec<RuntimeValue>,
    ) -> RuntimeResult {
        let mut environment = Environment::enclosed(Rc::clone(&interpreter.globals));

        for i in 0..self.declaration.params.len() {
            let param = &self.declaration.params[i].lexeme;
            let arg = std::mem::replace(&mut arguments[i], RuntimeValue::Nil);

            environment.define(param.clone(), arg);
        }

        interpreter.execute_block(
            &mut self.declaration.body,
            Rc::new(RefCell::new(environment)),
        )?;

        return Ok(RuntimeValue::Nil);
    }
}
