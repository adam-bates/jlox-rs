use crate::{
    interpreter::Interpreter,
    runtime_value::{RuntimeResult, RuntimeValue},
};

pub trait LoxCallable {
    fn arity(&self) -> usize;

    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<RuntimeValue>,
    ) -> RuntimeResult;
}
