use crate::{expr::*, runtime_value::RuntimeValue};

pub struct Interpreter {}

impl Visitor<RuntimeValue> for Interpreter {
    fn visit_literal_expr(&mut self, expr: &mut LiteralExpr) -> RuntimeValue {
        return RuntimeValue::from(&*expr);
    }

    fn visit_grouping_expr(&mut self, expr: &mut GroupingExpr) -> RuntimeValue {
        return self.evaluate(&mut expr.expr);
    }

    fn visit_unary_expr(&mut self, expr: &mut UnaryExpr) -> RuntimeValue {
        let right = self.evaluate(&mut expr.right);

        match expr.op.0 {
            UnaryExprOp::Not => RuntimeValue::Boolean(!self.is_truthy(&right)),

            UnaryExprOp::Minus => {
                let RuntimeValue::Number(value) = right else {
                    panic!("[{}:{}] Can't apply minus to expr: {right:#?}", file!(), line!());
                };

                return RuntimeValue::Number(-value);
            }
        }
    }

    fn visit_binary_expr(&mut self, expr: &mut BinaryExpr) -> RuntimeValue {
        let left = self.evaluate(&mut expr.left);
        let right = self.evaluate(&mut expr.right);

        match &expr.op.0 {
            BinaryExprOp::Plus => match (left, right) {
                (RuntimeValue::Number(left), RuntimeValue::Number(right)) => {
                    return RuntimeValue::Number(left + right);
                }
                (RuntimeValue::String(left), RuntimeValue::String(right)) => {
                    let mut res = left.to_string();
                    res.push_str(&right);
                    return RuntimeValue::String(res.into());
                }
                (left, right) => panic!(
                    "[{}:{}] Cannot add value: {left:#?}\nto value: {right:#?}",
                    file!(),
                    line!(),
                ),
            },

            BinaryExprOp::EqualEqual => RuntimeValue::Boolean(self.is_equal(&left, &right)),
            BinaryExprOp::NotEqual => RuntimeValue::Boolean(!self.is_equal(&left, &right)),

            op => {
                let RuntimeValue::Number(left) = left else {
                    panic!("[{}:{}] Expected type Number, found: {left:#?}", file!(), line!());
                };

                let RuntimeValue::Number(right) = right else {
                    panic!("[{}:{}] Expected type Number, found: {right:#?}", file!(), line!());
                };

                return match op {
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
                };
            }
        }
    }
}

impl Interpreter {
    fn evaluate(&mut self, expr: &mut Expr) -> RuntimeValue {
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
}
