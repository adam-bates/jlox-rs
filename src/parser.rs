use crate::{expr::*, lox, stmt::*, token::Token, token_type::TokenType};

pub type Result<T = ()> = std::result::Result<T, ParserError>;

pub struct ParserError {
    pub message: String,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        return Self { tokens, current: 0 };
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];

        while !self.is_at_end() {
            if let Some(statement) = self.declaration() {
                statements.push(statement);
            }
        }

        return statements;
    }

    fn declaration(&mut self) -> Option<Stmt> {
        fn try_declaration(this: &mut Parser) -> Result<Stmt> {
            if this.match_any(&[TokenType::Var]) {
                return this.var_declaration();
            } else {
                return this.statement();
            }
        }

        match try_declaration(self) {
            Ok(stmt) => return Some(stmt),
            Err(_e) => {
                self.synchronize();
                return None;
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(&TokenType::Identifier, "Expect variable name".to_string())?;

        let initializer = if self.match_any(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration.".to_string(),
        )?;

        return Ok(Stmt::Variable(VariableStmt { name, initializer }));
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_any(&[TokenType::Print]) {
            return self.print_statement();
        }

        return self.expression_statement();
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.".to_string())?;

        return Ok(Stmt::Print(PrintStmt(value)));
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after expression.".to_string(),
        )?;

        return Ok(Stmt::Expression(ExpressionStmt(expr)));
    }

    fn expression(&mut self) -> Result<Expr> {
        return self.equality();
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_any(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op_token = self.previous().unwrap().clone();

            let op = match op_token.token_type {
                TokenType::BangEqual => (BinaryExprOp::NotEqual, op_token),
                TokenType::EqualEqual => (BinaryExprOp::EqualEqual, op_token),
                _ => {
                    return Err(self.error(
                        format!("[{}:{}] Expected '!=' or '=='", file!(), line!()),
                        op_token,
                    ))
                }
            };

            let right = self.comparison()?;

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.match_any(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op_token = self.previous().unwrap().clone();

            let op = match op_token.token_type {
                TokenType::Greater => (BinaryExprOp::Greater, op_token),
                TokenType::GreaterEqual => (BinaryExprOp::GreaterEqual, op_token),
                TokenType::Less => (BinaryExprOp::Less, op_token),
                TokenType::LessEqual => (BinaryExprOp::LessEqual, op_token),
                _ => {
                    return Err(self.error(
                        format!("[{}:{}] Expected '>', '>=', '<', or '<='", file!(), line!()),
                        op_token,
                    ))
                }
            };

            let right = self.term()?;

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.match_any(&[TokenType::Minus, TokenType::Plus]) {
            let op_token = self.previous().unwrap().clone();

            let op = match op_token.token_type {
                TokenType::Minus => (BinaryExprOp::Minus, op_token),
                TokenType::Plus => (BinaryExprOp::Plus, op_token),
                _ => {
                    return Err(self.error(
                        format!("[{}:{}] Expected '+' or '-'", file!(), line!()),
                        op_token,
                    ))
                }
            };

            let right = self.factor()?;

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_any(&[TokenType::Slash, TokenType::Star]) {
            let op_token = self.previous().unwrap().clone();

            let op = match op_token.token_type {
                TokenType::Slash => (BinaryExprOp::Divide, op_token),
                TokenType::Star => (BinaryExprOp::Times, op_token),
                _ => {
                    return Err(self.error(
                        format!("[{}:{}] Expected '/' or '*'", file!(), line!()),
                        op_token,
                    ))
                }
            };

            let right = self.unary()?;

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_any(&[TokenType::Bang, TokenType::Minus]) {
            let op_token = self.previous().unwrap().clone();

            let op = match op_token.token_type {
                TokenType::Bang => (UnaryExprOp::Not, op_token),
                TokenType::Minus => (UnaryExprOp::Minus, op_token),
                _ => {
                    return Err(self.error(
                        format!("[{}:{}] Expected '!' or '-'", file!(), line!()),
                        op_token,
                    ))
                }
            };

            let right = self.unary()?;

            return Ok(Expr::Unary(UnaryExpr {
                op,
                right: Box::new(right),
            }));
        }

        return self.primary();
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.peek().unwrap().clone();

        if self.match_any(&[TokenType::False]) {
            return Ok(Expr::Literal(LiteralExpr(LiteralExprType::False, token)));
        };

        if self.match_any(&[TokenType::True]) {
            return Ok(Expr::Literal(LiteralExpr(LiteralExprType::True, token)));
        };

        if self.match_any(&[TokenType::Nil]) {
            return Ok(Expr::Literal(LiteralExpr(LiteralExprType::Nil, token)));
        };

        if self.match_any(&[
            TokenType::Number(Default::default()),
            TokenType::String(Default::default()),
        ]) {
            let literal_type = match token.token_type {
                TokenType::Number(_) => LiteralExprType::Number,
                TokenType::String(_) => LiteralExprType::String,
                _ => {
                    return Err(self.error(
                        format!("[{}:{}] Expected Number or String", file!(), line!()),
                        token,
                    ))
                }
            };

            return Ok(Expr::Literal(LiteralExpr(literal_type, token)));
        };

        if self.match_any(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(VariableExpr(token)));
        }

        if self.match_any(&[TokenType::LeftParen]) {
            let expr = self.expression()?;

            let right_token = self.consume(
                &TokenType::RightParen,
                format!("[{}:{}] Expected ')' after expression.", file!(), line!()),
            )?;

            return Ok(Expr::Grouping(GroupingExpr {
                left: token,
                expr: Box::new(expr),
                right: right_token,
            }));
        }

        if let Some(peek) = self.peek() {
            return Err(self.error(
                format!("[{}:{}] Expected some expression.", file!(), line!()),
                peek.clone(),
            ));
        } else {
            return Err(self.error(
                format!("[{}:{}] Unexpected end of file.", file!(), line!()),
                token,
            ));
        }
    }

    fn match_any(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        return false;
    }

    fn consume(&mut self, token_type: &TokenType, err_message: String) -> Result<Token> {
        if self.check(token_type) {
            return Ok(self.advance().unwrap().clone());
        }

        return Err(self.error(err_message, self.peek().unwrap().clone()));
    }

    fn error(&self, message: String, token: Token) -> ParserError {
        lox::token_error(token, &message);
        return ParserError { message };
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if let Some(prev) = self.previous() {
                if prev.token_type == TokenType::Semicolon {
                    return;
                }
            }

            if let Some(peek) = self.peek() {
                match peek.token_type {
                    TokenType::Class
                    | TokenType::For
                    | TokenType::Fun
                    | TokenType::If
                    | TokenType::Print
                    | TokenType::Return
                    | TokenType::Var
                    | TokenType::While => return,
                    _ => {}
                }
            }

            self.advance();
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        return self
            .peek()
            .map(|peek| {
                let mut peek = peek.clone();
                match peek.token_type {
                    TokenType::Number(_) => peek.token_type = TokenType::Number(Default::default()),
                    TokenType::String(_) => peek.token_type = TokenType::String(Default::default()),
                    _ => {}
                }

                return peek.token_type == *token_type;
            })
            .unwrap_or(false);
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }

        return self.previous();
    }

    fn is_at_end(&self) -> bool {
        return self
            .peek()
            .map(|peek| peek.token_type == TokenType::EOF)
            .unwrap_or(true);
    }

    fn peek(&self) -> Option<&Token> {
        return self.tokens.get(self.current);
    }

    fn previous(&self) -> Option<&Token> {
        if self.current == 0 {
            return None;
        }

        return self.tokens.get(self.current - 1);
    }
}
