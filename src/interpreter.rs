use crate::{
    expr::*,
    runtime_value::{RuntimeError, RuntimeResult, RuntimeValue},
    string::LoxStr, lox,
};

pub struct Interpreter;

impl Visitor<RuntimeResult> for Interpreter {
    fn visit_literal_expr(&mut self, expr: &mut LiteralExpr) -> RuntimeResult {
        return Ok(RuntimeValue::from(&*expr));
    }

    fn visit_grouping_expr(&mut self, expr: &mut GroupingExpr) -> RuntimeResult {
        return self.evaluate(&mut expr.expr);
    }

    fn visit_unary_expr(&mut self, expr: &mut UnaryExpr) -> RuntimeResult {
        let right = self.evaluate(&mut expr.right)?;

        match expr.op.0 {
            UnaryExprOp::Not => Ok(RuntimeValue::Boolean(!self.is_truthy(&right))),

            UnaryExprOp::Minus => {
                let RuntimeValue::Number(value) = right else {
                    return Err(RuntimeError::InvalidUnaryExpr {
                        expr: expr.clone(),
                        details: Some(format!("[{}:{}] Can only apply minus unary operator to numbers.", file!(), line!())),
                    });
                };

                return Ok(RuntimeValue::Number(-value));
            }
        }
    }

    fn visit_binary_expr(&mut self, expr: &mut BinaryExpr) -> RuntimeResult {
        let left = self.evaluate(&mut expr.left)?;
        let right = self.evaluate(&mut expr.right)?;

        match &expr.op.0 {
            BinaryExprOp::Plus => match (left, right) {
                (RuntimeValue::Number(left), RuntimeValue::Number(right)) => {
                    return Ok(RuntimeValue::Number(left + right));
                }
                (RuntimeValue::String(left), RuntimeValue::String(right)) => {
                    let mut res = left.to_string();
                    res.push_str(&right);
                    return Ok(RuntimeValue::String(res.into()));
                }
                (_left, _right) => {
                    return Err(RuntimeError::InvalidBinaryExpr {
                        expr: expr.clone(),
                        details: Some(format!(
                            "[{}:{}] Can only add 2 strings or 2 numbers.",
                            file!(),
                            line!()
                        )),
                    });
                }
            },

            BinaryExprOp::EqualEqual => Ok(RuntimeValue::Boolean(self.is_equal(&left, &right))),
            BinaryExprOp::NotEqual => Ok(RuntimeValue::Boolean(!self.is_equal(&left, &right))),

            op => {
                let RuntimeValue::Number(left) = left else {
                    return Err(RuntimeError::InvalidBinaryExpr {
                        expr: expr.clone(),
                        details: Some(format!(
                            "[{}:{}] Expected left operand to be a number.",
                            file!(),
                            line!()
                        )),
                    });
                };

                let RuntimeValue::Number(right) = right else {
                    return Err(RuntimeError::InvalidBinaryExpr {
                        expr: expr.clone(),
                        details: Some(format!(
                            "[{}:{}] Expected right operand to be a number.",
                            file!(),
                            line!()
                        )),
                    });
                };

                return Ok(match op {
                    BinaryExprOp::Plus | BinaryExprOp::EqualEqual | BinaryExprOp::NotEqual => {
                        unreachable!()
                    }

                    BinaryExprOp::Greater => RuntimeValue::Boolean(left > right),
                    BinaryExprOp::GreaterEqual => RuntimeValue::Boolean(left >= right),
                    BinaryExprOp::Less => RuntimeValue::Boolean(left < right),
                    BinaryExprOp::LessEqual => RuntimeValue::Boolean(left <= right),

                    BinaryExprOp::Minus => RuntimeValue::Number(left - right),
                    BinaryExprOp::Divide => RuntimeValue::Number(left / right),
                    BinaryExprOp::Times => RuntimeValue::Number(left * right),
                });
            }
        }
    }
}

impl Interpreter {
    pub fn interpret(&mut self, expr: &mut Expr) {
        match self.evaluate(expr) {
            Ok(value) => {
                println!("{}", self.stringify(&value));
            }
            Err(e) => {
                lox::runtime_error(e);
            }
        }
    }

    fn evaluate(&mut self, expr: &mut Expr) -> RuntimeResult {
        return expr.accept(self);
    }

    fn is_truthy(&self, value: &RuntimeValue) -> bool {
        if let RuntimeValue::Nil = value {
            return false;
        }

        if let RuntimeValue::Boolean(value) = value {
            return *value;
        }

        return true;
    }

    fn is_equal(&self, left: &RuntimeValue, right: &RuntimeValue) -> bool {
        return left == right;
    }

    fn stringify(&self, value: &RuntimeValue) -> LoxStr {
        match value {
            RuntimeValue::Nil => return "nil".into(),

            RuntimeValue::Number(value) => {
                let mut text = value.to_string();

                if text.ends_with(".0") {
                    text.pop(); // 123.0 -> 123.
                    text.pop(); // 123.  -> 123
                }

                return text.into();
            }

            RuntimeValue::String(value) => return value.clone(),

            RuntimeValue::Boolean(value) => return value.to_string().into(),

            RuntimeValue::Object(value) => return self.stringify(value),
        }
    }
}