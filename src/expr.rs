use crate::token::Token;

// Manually writing this part out
// as it seems easier than translating the Java generation code

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(LiteralExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Variable(VariableExpr),
    Assignment(AssignmentExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteralExpr(pub LiteralExprType, pub Token);

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralExprType {
    Number,
    String,
    True,
    False,
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub op: (UnaryExprOp, Token),
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryExprOp {
    Minus,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: (BinaryExprOp, Token),
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryExprOp {
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Times,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupingExpr {
    pub left: Token,
    pub expr: Box<Expr>,
    pub right: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableExpr(pub Token);

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpr {
    pub name: Token,
    pub value: Box<Expr>,
}

// Visitor pattern
pub trait ExprVisitor<R> {
    fn visit_literal_expr(&mut self, expr: &mut LiteralExpr) -> R;
    fn visit_unary_expr(&mut self, expr: &mut UnaryExpr) -> R;
    fn visit_binary_expr(&mut self, expr: &mut BinaryExpr) -> R;
    fn visit_grouping_expr(&mut self, expr: &mut GroupingExpr) -> R;
    fn visit_variable_expr(&mut self, expr: &mut VariableExpr) -> R;
    fn visit_assignment_expr(&mut self, expr: &mut AssignmentExpr) -> R;
}

pub trait ExprAccept<R, V: ExprVisitor<R>> {
    fn accept(&mut self, visitor: &mut V) -> R;
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for Expr {
    fn accept(&mut self, visitor: &mut V) -> R {
        return match self {
            Self::Literal(expr) => expr.accept(visitor),
            Self::Unary(expr) => expr.accept(visitor),
            Self::Binary(expr) => expr.accept(visitor),
            Self::Grouping(expr) => expr.accept(visitor),
            Self::Variable(expr) => expr.accept(visitor),
            Self::Assignment(expr) => expr.accept(visitor),
        };
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for LiteralExpr {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_literal_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for UnaryExpr {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_unary_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for BinaryExpr {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_binary_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for GroupingExpr {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_grouping_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for VariableExpr {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_variable_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for AssignmentExpr {
    fn accept(&mut self, visitor: &mut V) -> R {
        return visitor.visit_assignment_expr(self);
    }
}
