use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::stmt::FunctionStmt,
    environment::Environment,
    interpreter::Interpreter,
    lox_callable::LoxCall,
    lox_instance::LoxInstance,
    runtime_value::{RuntimeError, RuntimeResult, RuntimeValue},
    string::LoxStr,
    token::Token,
    token_type::TokenType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    pub declaration: FunctionStmt,
    pub closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        declaration: FunctionStmt,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        return Self {
            declaration,
            closure,
            is_initializer,
        };
    }

    pub fn bind(&self, instance: LoxInstance) -> Self {
        let mut environment = Environment::enclosed(Rc::clone(&self.closure));
        environment.define("this".into(), RuntimeValue::LoxInstance(instance));
        return Self::new(
            self.declaration.clone(),
            Rc::new(RefCell::new(environment)),
            self.is_initializer,
        );
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

        if let Err(e) = interpreter.execute_block(
            &mut self.declaration.body,
            Rc::new(RefCell::new(environment)),
        ) {
            match e {
                RuntimeError::NonErrorReturnShortCircuit { value } => {
                    if self.is_initializer {
                        return Environment::get_at(
                            Rc::clone(&self.closure),
                            0,
                            &Token {
                                token_type: TokenType::This,
                                lexeme: "this".into(),
                                line: 0,
                            },
                        );
                    }

                    return Ok(value.unwrap_or_else(|| RuntimeValue::Nil));
                }
                e => return Err(e),
            }
        }

        if self.is_initializer {
            return Environment::get_at(
                Rc::clone(&self.closure),
                0,
                &Token {
                    token_type: TokenType::This,
                    lexeme: "this".into(),
                    line: 0,
                },
            );
        }

        return Ok(RuntimeValue::Nil);
    }

    fn to_string(&self) -> LoxStr {
        return format!("<fn {}>", self.declaration.name.lexeme).into();
    }
}
