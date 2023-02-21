use crate::token::Token;

// Manually writing this part out
// as it seems easier than translating the Java generation code

static mut NEXT_EXPR_ID: usize = 0;

pub fn expr_id() -> usize {
    let id = unsafe { NEXT_EXPR_ID };
    unsafe { NEXT_EXPR_ID += 1 };
    return id;
}

pub type ExprId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(LiteralExpr),
    Logical(LogicalExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    Grouping(GroupingExpr),
    Variable(VariableExpr),
    Assignment(AssignmentExpr),
    Get(GetExpr),
    Set(SetExpr),
    This(ThisExpr),
}

impl Expr {
    pub fn id(&self) -> usize {
        return match self {
            Self::Literal(expr) => expr.id,
            Self::Logical(expr) => expr.id,
            Self::Unary(expr) => expr.id,
            Self::Binary(expr) => expr.id,
            Self::Call(expr) => expr.id,
            Self::Grouping(expr) => expr.id,
            Self::Variable(expr) => expr.id,
            Self::Assignment(expr) => expr.id,
            Self::Get(expr) => expr.id,
            Self::Set(expr) => expr.id,
            Self::This(expr) => expr.id,
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteralExpr {
    pub id: ExprId,
    pub literal_type: LiteralExprType,
    pub token: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralExprType {
    Number,
    String,
    True,
    False,
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalExpr {
    pub id: ExprId,
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub id: ExprId,
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
    pub id: ExprId,
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
pub struct CallExpr {
    pub id: ExprId,
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupingExpr {
    pub id: ExprId,
    pub left: Token,
    pub expr: Box<Expr>,
    pub right: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableExpr {
    pub id: ExprId,
    pub name: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpr {
    pub id: ExprId,
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetExpr {
    pub id: ExprId,
    pub object: Box<Expr>,
    pub name: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetExpr {
    pub id: ExprId,
    pub object: Box<Expr>,
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThisExpr {
    pub id: ExprId,
    pub keyword: Token,
}

// Visitor pattern
pub trait ExprVisitor<R> {
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> R;
    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> R;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> R;
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> R;
    fn visit_call_expr(&mut self, expr: &CallExpr) -> R;
    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> R;
    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> R;
    fn visit_assignment_expr(&mut self, expr: &AssignmentExpr) -> R;
    fn visit_get_expr(&mut self, expr: &GetExpr) -> R;
    fn visit_set_expr(&mut self, expr: &SetExpr) -> R;
    fn visit_this_expr(&mut self, expr: &ThisExpr) -> R;
}

pub trait ExprAccept<R, V: ExprVisitor<R>> {
    fn accept(&self, visitor: &mut V) -> R;
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for Expr {
    fn accept(&self, visitor: &mut V) -> R {
        return match self {
            Self::Literal(expr) => expr.accept(visitor),
            Self::Logical(expr) => expr.accept(visitor),
            Self::Unary(expr) => expr.accept(visitor),
            Self::Binary(expr) => expr.accept(visitor),
            Self::Call(expr) => expr.accept(visitor),
            Self::Grouping(expr) => expr.accept(visitor),
            Self::Variable(expr) => expr.accept(visitor),
            Self::Assignment(expr) => expr.accept(visitor),
            Self::Get(expr) => expr.accept(visitor),
            Self::Set(expr) => expr.accept(visitor),
            Self::This(expr) => expr.accept(visitor),
        };
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for LiteralExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_literal_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for LogicalExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_logical_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for UnaryExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_unary_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for BinaryExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_binary_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for CallExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_call_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for GroupingExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_grouping_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for VariableExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_variable_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for AssignmentExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_assignment_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for GetExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_get_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for SetExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_set_expr(self);
    }
}

impl<R, V: ExprVisitor<R>> ExprAccept<R, V> for ThisExpr {
    fn accept(&self, visitor: &mut V) -> R {
        return visitor.visit_this_expr(self);
    }
}
