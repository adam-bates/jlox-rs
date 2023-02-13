use crate::{expr::Expr, token::Token};

// Manually writing this part out
// as it seems easier than translating the Java generation code

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block(BlockStmt),
    Expression(ExpressionStmt),
    Print(PrintStmt),
    Variable(VariableStmt),
    If(IfStmt),
    While(WhileStmt),
    Function(FunctionStmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt(pub Vec<Stmt>);

#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionStmt(pub Expr);

#[derive(Debug, Clone, PartialEq)]
pub struct PrintStmt(pub Expr);

#[derive(Debug, Clone, PartialEq)]
pub struct VariableStmt {
    pub name: Token,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

// Visitor pattern
pub trait StmtVisitor<R> {
    fn visit_block_stmt(&mut self, stmt: &mut BlockStmt) -> R;
    fn visit_expression_stmt(&mut self, stmt: &mut ExpressionStmt) -> R;
    fn visit_print_stmt(&mut self, stmt: &mut PrintStmt) -> R;
    fn visit_variable_stmt(&mut self, stmt: &mut VariableStmt) -> R;
    fn visit_if_stmt(&mut self, stmt: &mut IfStmt) -> R;
    fn visit_while_stmt(&mut self, stmt: &mut WhileStmt) -> R;
    fn visit_function_stmt(&mut self, stmt: &mut FunctionStmt) -> R;
}

pub trait StmtAccept<R, V: StmtVisitor<R>> {
    fn accept(&mut self, visitor: &mut V) -> R;
}

impl<R, V: StmtVisitor<R>> StmtAccept<R, V> for Stmt {
    fn accept(&mut self, visitor: &mut V) -> R {
        return match self {
            Self::Block(stmt) => stmt.accept(visitor),
            Self::Expression(stmt) => stmt.accept(visitor),
            Self::Print(stmt) => stmt.accept(visitor),
            Self::Variable(stmt) => stmt.accept(visitor),
            Self::If(stmt) => stmt.accept(visitor),
            Self::While(stmt) => stmt.accept(visitor),
            Self::Function(stmt) => stmt.accept(visitor),
        };
    }
}

impl<R, V: StmtVisitor<R>> StmtAccept<R, V> for BlockStmt {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_block_stmt(self);
    }
}

impl<R, V: StmtVisitor<R>> StmtAccept<R, V> for ExpressionStmt {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_expression_stmt(self);
    }
}

impl<R, V: StmtVisitor<R>> StmtAccept<R, V> for PrintStmt {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_print_stmt(self);
    }
}

impl<R, V: StmtVisitor<R>> StmtAccept<R, V> for VariableStmt {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_variable_stmt(self);
    }
}

impl<R, V: StmtVisitor<R>> StmtAccept<R, V> for IfStmt {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_if_stmt(self);
    }
}

impl<R, V: StmtVisitor<R>> StmtAccept<R, V> for WhileStmt {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_while_stmt(self);
    }
}

impl<R, V: StmtVisitor<R>> StmtAccept<R, V> for FunctionStmt {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_function_stmt(self);
    }
}
