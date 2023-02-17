use crate::{
    interpreter::Interpreter,
    lox_class::LoxClass,
    lox_function::LoxFunction,
    runtime_value::{RuntimeResult, RuntimeValue},
    string::LoxStr,
};

pub trait LoxCall {
    fn arity(&self) -> usize;
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<RuntimeValue>,
    ) -> RuntimeResult;
    fn to_string(&self) -> LoxStr;
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoxCallable {
    LoxFunction(LoxFunction),
    LoxClass(LoxClass),
    Clock(Clock),
}

impl LoxCall for LoxCallable {
    fn arity(&self) -> usize {
        return match self {
            Self::LoxFunction(function) => function.arity(),
            Self::LoxClass(class) => class.arity(),
            Self::Clock(clock) => clock.arity(),
        };
    }

    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<RuntimeValue>,
    ) -> RuntimeResult {
        return match self {
            Self::LoxFunction(function) => function.call(interpreter, arguments),
            Self::LoxClass(class) => class.call(interpreter, arguments),
            Self::Clock(clock) => clock.call(interpreter, arguments),
        };
    }

    fn to_string(&self) -> LoxStr {
        return match self {
            Self::LoxFunction(function) => function.to_string(),
            Self::LoxClass(class) => class.to_string(),
            Self::Clock(clock) => clock.to_string(),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clock;
impl LoxCall for Clock {
    fn arity(&self) -> usize {
        return 0;
    }

    fn call(&mut self, _: &mut Interpreter, _: Vec<RuntimeValue>) -> RuntimeResult {
        use std::time::SystemTime;

        let epoch_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        return Ok(RuntimeValue::Number(epoch_time.as_millis() as f64 / 1000.0));
    }

    fn to_string(&self) -> LoxStr {
        return "<fn clock>".into();
    }
}
