use crate::{
    interpreter::Interpreter,
    lox_callable::LoxCall,
    lox_instance::LoxInstance,
    runtime_value::{RuntimeResult, RuntimeValue},
    string::LoxStr,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    pub name: LoxStr,
}

impl LoxClass {
    pub fn new(name: LoxStr) -> Self {
        return Self { name };
    }
}

impl LoxCall for LoxClass {
    fn arity(&self) -> usize {
        return 0;
    }

    fn call(
        &mut self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<RuntimeValue>,
    ) -> RuntimeResult {
        let instance = LoxInstance::new(self.clone());

        return Ok(RuntimeValue::LoxInstance(instance));
    }

    fn to_string(&self) -> LoxStr {
        return self.name.clone();
    }
}
