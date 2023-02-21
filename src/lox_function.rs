use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::stmt::FunctionStmt,
    environment::Environment,
    interpreter::Interpreter,
    lox_callable::LoxCall,
    lox_instance::LoxInstance,
    runtime_value::{RuntimeError, RuntimeResult, RuntimeValue},
    string::LoxStr,
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

    pub fn bind(&self, instance: LoxInstance) -> Self {
        let mut environment = Environment::enclosed(Rc::clone(&self.closure));
        environment.define("this".into(), RuntimeValue::LoxInstance(instance));
        return Self::new(self.declaration.clone(), Rc::new(RefCell::new(environment)));
    }
}

impl LoxCall for LoxFunction {
    fn arity(&self) -> usize {
        return self.declaration.params.len();
    }

    fn call(
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

    fn to_string(&self) -> LoxStr {
        return format!("<fn {}>", self.declaration.name.lexeme).into();
    }
}
