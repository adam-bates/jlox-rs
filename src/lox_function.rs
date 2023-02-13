use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    interpreter::Interpreter,
    runtime_value::{RuntimeError, RuntimeResult, RuntimeValue},
    stmt::FunctionStmt,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    pub declaration: FunctionStmt,
    pub closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(declaration: FunctionStmt, closure: Rc<RefCell<Environment>>) -> Self {
        return Self {
            declaration,
            closure,
        };
    }

    pub fn arity(&self) -> usize {
        return self.declaration.params.len();
    }

    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        mut arguments: Vec<RuntimeValue>,
    ) -> RuntimeResult {
        let mut environment = Environment::enclosed(Rc::clone(&self.closure));

        for i in 0..self.declaration.params.len() {
            let param = &self.declaration.params[i].lexeme;
            let arg = std::mem::replace(&mut arguments[i], RuntimeValue::Nil);

            environment.define(param.clone(), arg);
        }

        match interpreter.execute_block(
            &mut self.declaration.body,
            Rc::new(RefCell::new(environment)),
        ) {
            Ok(()) => {
                return Ok(RuntimeValue::Nil);
            }
            Err(RuntimeError::NonErrorReturnShortCircuit { value }) => {
                return Ok(value.unwrap_or_else(|| RuntimeValue::Nil));
            }
            Err(e) => return Err(e),
        }
    }
}
