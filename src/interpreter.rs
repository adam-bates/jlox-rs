use crate::{
    environment::Environment,
    expr::*,
    lox,
    runtime_value::{RuntimeError, RuntimeResult, RuntimeValue},
    stmt::*,
    string::LoxStr,
};

use std::{cell::RefCell, rc::Rc};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        return Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        };
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for mut statement in statements {
            match self.execute(&mut statement) {
                Err(e) => {
                    lox::runtime_error(e);
                    break;
                }
                _ => {}
            }
        }
    }

    fn execute(&mut self, stmt: &mut Stmt) -> RuntimeResult<()> {
        return stmt.accept(self);
    }

    fn evaluate(&mut self, expr: &mut Expr) -> RuntimeResult {
        return expr.accept(self);
    }

    fn execute_block(
        &mut self,
        statements: &mut Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> RuntimeResult<()> {
        let previous = Rc::clone(&self.environment);

        self.environment = environment;

        let try_execute_block = || -> RuntimeResult<()> {
            for statement in statements {
                self.execute(statement)?;
            }

            return Ok(());
        };

        let res = try_execute_block();

        self.environment = previous;

        return res;
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

impl ExprVisitor<RuntimeResult> for Interpreter {
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

    fn visit_variable_expr(&mut self, expr: &mut VariableExpr) -> RuntimeResult {
        return self.environment.borrow().get(&expr.0);
    }

    fn visit_assignment_expr(&mut self, expr: &mut AssignmentExpr) -> RuntimeResult {
        let value = self.evaluate(&mut expr.value)?;

        self.environment
            .borrow_mut()
            .assign(expr.name.clone(), value.clone())?;

        return Ok(value);
    }
}

impl StmtVisitor<RuntimeResult<()>> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &mut ExpressionStmt) -> RuntimeResult<()> {
        self.evaluate(&mut stmt.0)?;

        return Ok(());
    }

    fn visit_print_stmt(&mut self, stmt: &mut PrintStmt) -> RuntimeResult<()> {
        let value = self.evaluate(&mut stmt.0)?;

        println!("{}", self.stringify(&value));

        return Ok(());
    }

    fn visit_variable_stmt(&mut self, stmt: &mut VariableStmt) -> RuntimeResult<()> {
        let value = if let Some(initializer) = &mut stmt.initializer {
            self.evaluate(initializer)?
        } else {
            RuntimeValue::Nil
        };

        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), value);

        return Ok(());
    }

    fn visit_block_stmt(&mut self, stmt: &mut BlockStmt) -> RuntimeResult<()> {
        self.execute_block(
            &mut stmt.0,
            Rc::new(RefCell::new(Environment::enclosed(Rc::clone(
                &self.environment,
            )))),
        )?;

        return Ok(());
    }

    fn visit_if_stmt(&mut self, stmt: &mut IfStmt) -> RuntimeResult<()> {
        let mut condition = self.evaluate(&mut stmt.condition)?;

        if self.is_truthy(&mut condition) {
            self.execute(&mut stmt.then_branch)?;
        } else if let Some(else_branch) = &mut stmt.else_branch {
            self.execute(else_branch)?;
        }

        return Ok(());
    }
}
