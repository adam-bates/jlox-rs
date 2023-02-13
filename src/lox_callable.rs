use crate::{
    interpreter::Interpreter,
    lox_function::LoxFunction,
    runtime_value::{RuntimeResult, RuntimeValue},
};

#[derive(Debug, Clone, PartialEq)]
pub enum LoxCallable {
    LoxFunction(LoxFunction),
    Clock(Clock),
}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        return match self {
            Self::LoxFunction(function) => function.arity(),
            Self::Clock(clock) => clock.arity(),
        };
    }

    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<RuntimeValue>,
    ) -> RuntimeResult {
        return match self {
            Self::LoxFunction(function) => function.call(interpreter, arguments),
            Self::Clock(clock) => clock.call(interpreter, arguments),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clock;
impl Clock {
    pub fn arity(&self) -> usize {
        return 0;
    }

    pub fn call(&mut self, _: &mut Interpreter, _: Vec<RuntimeValue>) -> RuntimeResult {
        use std::time::SystemTime;

        let epoch_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        return Ok(RuntimeValue::Number(epoch_time.as_millis() as f64 / 1000.0));
    }
}
