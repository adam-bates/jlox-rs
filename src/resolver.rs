use std::collections::HashMap;

use crate::{
    ast::{expr::*, stmt::*},
    interpreter::Interpreter,
    lox,
    string::LoxStr,
    token::Token,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<LoxStr, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        return Self {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        };
    }

    pub fn resolve(&mut self, statements: &Vec<Stmt>) {
        self.resolve_stmts(statements);
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                lox::token_error(
                    name.clone(),
                    "Already a variable with this name in this scope",
                );
            }

            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_stmts(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        stmt.accept(self);
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        expr.accept(self);
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        if self.scopes.len() == 0 {
            return;
        }

        let mut i = self.scopes.len() - 1;

        loop {
            if self.scopes[i].contains_key(&name.lexeme) {
                self.interpreter
                    .resolve(expr.id(), self.scopes.len() - 1 - i);
                return;
            }

            if i == 0 {
                break;
            }

            i -= 1;
        }
    }

    fn resolve_function(&mut self, function: &FunctionStmt, function_type: FunctionType) {
        let enclosing_function = self.current_function;
        self.current_function = function_type;

        self.begin_scope();

        for param in &function.params {
            self.declare(param);
            self.define(param);
        }

        self.resolve_stmts(&function.body);

        self.end_scope();

        self.current_function = enclosing_function;
    }
}

impl ExprVisitor<()> for Resolver<'_> {
    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> () {
        if let Some(scope) = self.scopes.last() {
            if let Some(false) = scope.get(&expr.name.lexeme) {
                lox::token_error(
                    expr.name.clone(),
                    "Can't read local variable in its own initializer",
                );
            }
        }

        self.resolve_local(&Expr::Variable(expr.clone()), &expr.name);
    }

    fn visit_assignment_expr(&mut self, expr: &AssignmentExpr) -> () {
        self.resolve_expr(&expr.value);
        self.resolve_local(&Expr::Assignment(expr.clone()), &expr.name);
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> () {
        self.resolve_expr(&expr.left);
        self.resolve_expr(&expr.right);
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) -> () {
        self.resolve_expr(&expr.callee);

        for argument in &expr.arguments {
            self.resolve_expr(argument);
        }
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> () {
        self.resolve_expr(&expr.expr);
    }

    fn visit_literal_expr(&mut self, _: &LiteralExpr) -> () {
        // NO-OP
    }

    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> () {
        self.resolve_expr(&expr.left);
        self.resolve_expr(&expr.right);
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> () {
        self.resolve_expr(&expr.right);
    }

    fn visit_get_expr(&mut self, expr: &GetExpr) -> () {
        self.resolve_expr(&expr.object);
    }

    fn visit_set_expr(&mut self, expr: &SetExpr) -> () {
        self.resolve_expr(&expr.value);
        self.resolve_expr(&expr.object);
    }

    fn visit_this_expr(&mut self, expr: &ThisExpr) -> () {
        if self.current_class == ClassType::None {
            lox::token_error(expr.keyword.clone(), "Can't use 'this' outside of a class");
            return;
        }

        self.resolve_local(&Expr::This(expr.clone()), &expr.keyword);
    }
}

impl StmtVisitor<()> for Resolver<'_> {
    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> () {
        self.begin_scope();

        self.resolve_stmts(&stmt.stmts);

        self.end_scope();
    }

    fn visit_variable_stmt(&mut self, stmt: &VariableStmt) -> () {
        self.declare(&stmt.name);

        if let Some(initializer) = &stmt.initializer {
            self.resolve_expr(initializer);
        }

        self.define(&stmt.name);
    }

    fn visit_function_stmt(&mut self, stmt: &FunctionStmt) -> () {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        self.resolve_function(stmt, FunctionType::Function);
    }

    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> () {
        self.resolve_expr(&stmt.expr);
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> () {
        self.resolve_expr(&stmt.condition);
        self.resolve_stmt(&stmt.then_branch);

        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_stmt(else_branch);
        }
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt) -> () {
        self.resolve_expr(&stmt.expr);
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> () {
        if self.current_function == FunctionType::None {
            lox::token_error(stmt.keyword.clone(), "Can't return from top-level code");
        }

        if let Some(value) = &stmt.value {
            if self.current_function == FunctionType::Initializer {
                lox::token_error(
                    stmt.keyword.clone(),
                    "Can't return a value from an initializer",
                );
            }

            self.resolve_expr(value);
        }
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> () {
        self.resolve_expr(&stmt.condition);
        self.resolve_stmt(&stmt.body);
    }

    fn visit_class_stmt(&mut self, stmt: &ClassStmt) -> () {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(&stmt.name);
        self.define(&stmt.name);

        self.begin_scope();
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert("this".into(), true);
        }

        for method in &stmt.methods {
            let declaration = match method.name.lexeme.as_ref() {
                "init" => FunctionType::Initializer,
                _ => FunctionType::Method,
            };

            self.resolve_function(method, declaration);
        }

        self.end_scope();

        self.current_class = enclosing_class;
    }
}
