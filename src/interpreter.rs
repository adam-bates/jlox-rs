use crate::{
    ast::{expr::*, stmt::*},
    environment::Environment,
    lox,
    lox_callable::{Clock, LoxCall, LoxCallable},
    lox_class::LoxClass,
    lox_function::LoxFunction,
    runtime_value::{RuntimeError, RuntimeResult, RuntimeValue},
    string::LoxStr,
    token::Token,
    token_type::TokenType,
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,

    environment: Rc<RefCell<Environment>>,
    locals: HashMap<ExprId, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));

        globals.borrow_mut().define(
            "clock".into(),
            RuntimeValue::LoxCallable(LoxCallable::Clock(Clock)),
        );

        return Self {
            environment: Rc::clone(&globals),
            globals,

            locals: HashMap::new(),
        };
    }

    pub fn resolve(&mut self, id: ExprId, depth: usize) {
        self.locals.insert(id, depth);
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

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> RuntimeResult<()> {
        let previous = Rc::clone(&self.environment);

        self.environment = environment;

        let mut try_execute_block = || -> RuntimeResult<()> {
            for statement in statements {
                self.execute(statement)?;
            }

            return Ok(());
        };

        let res = try_execute_block();

        self.environment = previous;

        return res;
    }

    fn look_up_variable(&self, name: &Token, expr_id: &ExprId) -> RuntimeResult {
        if let Some(distance) = self.locals.get(expr_id) {
            return Environment::get_at(Rc::clone(&self.environment), *distance, name);
        } else {
            return self.globals.borrow().get(name);
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> RuntimeResult<()> {
        return stmt.accept(self);
    }

    fn evaluate(&mut self, expr: &Expr) -> RuntimeResult {
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

            RuntimeValue::LoxCallable(callable) => {
                return format!("{}", callable.to_string()).into()
            }

            RuntimeValue::LoxInstance(instance) => {
                return format!("{} instance", instance.class.name).into()
            }
        }
    }
}

impl ExprVisitor<RuntimeResult> for Interpreter {
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> RuntimeResult {
        return Ok(RuntimeValue::from(expr));
    }

    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> RuntimeResult {
        let left = self.evaluate(&expr.left)?;

        if expr.operator.token_type == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !self.is_truthy(&left) {
                return Ok(left);
            }
        }

        return self.evaluate(&expr.right);
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> RuntimeResult {
        return self.evaluate(&expr.expr);
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> RuntimeResult {
        let right = self.evaluate(&expr.right)?;

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

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> RuntimeResult {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

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

    fn visit_call_expr(&mut self, expr: &CallExpr) -> RuntimeResult {
        let callee = self.evaluate(&expr.callee)?;

        let mut arguments = vec![];
        for argument in &expr.arguments {
            arguments.push(self.evaluate(argument)?);
        }

        let RuntimeValue::LoxCallable(mut function) = callee else {
            return Err(RuntimeError::InvalidCallable {
                value: callee,
                details: Some("Can only call functions and classes".to_string()),
            });
        };

        if arguments.len() != function.arity() {
            return Err(RuntimeError::WrongNumberOfArgs {
                expected: function.arity(),
                found: arguments.len(),
                details: Some(format!("Expr: {expr:?}")),
            });
        }

        return function.call(self, arguments);
    }

    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> RuntimeResult {
        // return self.environment.borrow().get(&expr.name);
        return self.look_up_variable(&expr.name, &expr.id);
    }

    fn visit_assignment_expr(&mut self, expr: &AssignmentExpr) -> RuntimeResult {
        let value = self.evaluate(&expr.value)?;

        if let Some(distance) = self.locals.get(&expr.id) {
            Environment::assign_at(
                Rc::clone(&self.environment),
                *distance,
                expr.name.clone(),
                value.clone(),
            )?;
        } else {
            self.globals
                .borrow_mut()
                .assign(expr.name.clone(), value.clone())?;
        }

        return Ok(value);
    }

    fn visit_get_expr(&mut self, expr: &GetExpr) -> RuntimeResult {
        let object = self.evaluate(&expr.object)?;

        if let RuntimeValue::LoxInstance(instance) = object {
            return instance.get(&expr.name);
        }

        return Err(RuntimeError::InvalidGetExpr {
            name: expr.name.clone(),
            details: Some("Only instances have properties".to_string()),
        });
    }

    fn visit_set_expr(&mut self, expr: &SetExpr) -> RuntimeResult {
        let object = self.evaluate(&expr.object)?;

        let RuntimeValue::LoxInstance(mut instance) = object else {
            return Err(RuntimeError::InvalidSetExpr {
                name: expr.name.clone(),
                details: Some("Only instances have fields".to_string()),
            });
        };

        let value = self.evaluate(&expr.value)?;

        instance.set(expr.name.clone(), value.clone());

        return Ok(value);
    }

    fn visit_this_expr(&mut self, expr: &ThisExpr) -> RuntimeResult {
        return self.look_up_variable(&expr.keyword, &expr.id);
    }
}

impl StmtVisitor<RuntimeResult<()>> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> RuntimeResult<()> {
        self.evaluate(&stmt.expr)?;

        return Ok(());
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt) -> RuntimeResult<()> {
        let value = self.evaluate(&stmt.expr)?;

        println!("{}", self.stringify(&value));

        return Ok(());
    }

    fn visit_variable_stmt(&mut self, stmt: &VariableStmt) -> RuntimeResult<()> {
        let value = if let Some(initializer) = &stmt.initializer {
            self.evaluate(initializer)?
        } else {
            RuntimeValue::Nil
        };

        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), value);

        return Ok(());
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> RuntimeResult<()> {
        self.execute_block(
            &stmt.stmts,
            Rc::new(RefCell::new(Environment::enclosed(Rc::clone(
                &self.environment,
            )))),
        )?;

        return Ok(());
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> RuntimeResult<()> {
        let mut condition = self.evaluate(&stmt.condition)?;

        if self.is_truthy(&mut condition) {
            self.execute(&stmt.then_branch)?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)?;
        }

        return Ok(());
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> RuntimeResult<()> {
        while {
            let condition = self.evaluate(&stmt.condition)?;
            self.is_truthy(&condition)
        } {
            self.execute(&stmt.body)?;
        }

        return Ok(());
    }

    fn visit_function_stmt(&mut self, stmt: &FunctionStmt) -> RuntimeResult<()> {
        let name = stmt.name.lexeme.clone();

        let function = LoxFunction::new(stmt.clone(), Rc::clone(&self.environment), false);

        self.environment.borrow_mut().define(
            name,
            RuntimeValue::LoxCallable(LoxCallable::LoxFunction(function)),
        );

        return Ok(());
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> RuntimeResult<()> {
        let value = if let Some(value) = &stmt.value {
            Some(self.evaluate(value)?)
        } else {
            None
        };

        return Err(RuntimeError::NonErrorReturnShortCircuit { value });
    }

    fn visit_class_stmt(&mut self, stmt: &ClassStmt) -> RuntimeResult<()> {
        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), RuntimeValue::Nil);

        let mut methods = HashMap::new();
        for method in &stmt.methods {
            let function = LoxFunction::new(
                method.clone(),
                Rc::clone(&self.environment),
                method.name.lexeme == "init",
            );

            methods.insert(method.name.lexeme.clone(), function);
        }

        let class = LoxClass::new(stmt.name.lexeme.clone(), Rc::new(RefCell::new(methods)));
        self.environment.borrow_mut().assign(
            stmt.name.clone(),
            RuntimeValue::LoxCallable(LoxCallable::LoxClass(class)),
        )?;

        return Ok(());
    }
}
