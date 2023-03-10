use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    interpreter::Interpreter,
    lox_callable::LoxCall,
    lox_function::LoxFunction,
    lox_instance::LoxInstance,
    runtime_value::{RuntimeResult, RuntimeValue},
    string::LoxStr,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    pub name: LoxStr,
    pub methods: Rc<RefCell<HashMap<LoxStr, LoxFunction>>>,
}

impl LoxClass {
    pub fn new(name: LoxStr, methods: Rc<RefCell<HashMap<LoxStr, LoxFunction>>>) -> Self {
        return Self { name, methods };
    }

    pub fn find_method<'a>(
        methods: &'a Ref<HashMap<LoxStr, LoxFunction>>,
        name: &LoxStr,
    ) -> Option<&'a LoxFunction> {
        return methods.get(name);
    }
}

impl LoxCall for LoxClass {
    fn arity(&self) -> usize {
        match Self::find_method(&self.methods.borrow(), &"init".into()) {
            Some(initializer) => return initializer.arity(),
            None => return 0,
        }
    }

    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<RuntimeValue>,
    ) -> RuntimeResult {
        let instance = LoxInstance::new(self.clone());

        if let Some(initializer) = Self::find_method(&self.methods.borrow(), &"init".into()) {
            initializer
                .bind(instance.clone())
                .call(interpreter, arguments)?;
        }

        return Ok(RuntimeValue::LoxInstance(instance));
    }

    fn to_string(&self) -> LoxStr {
        return self.name.clone();
    }
}
