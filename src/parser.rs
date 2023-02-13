use crate::{expr::*, lox, stmt::*, string::LoxStr, token::Token, token_type::TokenType};

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
            if this.match_any(&[TokenType::Fun]) {
                return Ok(Stmt::Function(this.function("function".into())?));
            }

            if this.match_any(&[TokenType::Var]) {
                return this.var_declaration();
            }

            return this.statement();
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
        if self.match_any(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.match_any(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.match_any(&[TokenType::For]) {
            return self.for_statement();
        }

        if self.match_any(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.match_any(&[TokenType::Return]) {
            return self.return_statement();
        }

        if self.match_any(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(BlockStmt(self.block()?)));
        }

        return self.expression_statement();
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'if'".to_string())?;
        let condition = self.expression()?;
        self.consume(
            &TokenType::RightParen,
            "Expect ')' after if condition".to_string(),
        )?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_any(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        return Ok(Stmt::If(IfStmt {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        }));
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(
            &TokenType::LeftParen,
            "Expect '(' after 'while'".to_string(),
        )?;
        let condition = self.expression()?;
        self.consume(
            &TokenType::RightParen,
            "Expect ')' after while condition".to_string(),
        )?;

        let body = self.statement()?;

        return Ok(Stmt::While(WhileStmt {
            condition,
            body: Box::new(body),
        }));
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'for'".to_string())?;

        let initializer = if self.match_any(&[TokenType::Semicolon]) {
            None
        } else if self.match_any(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after for clauses".to_string(),
        )?;

        let increment = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            &TokenType::RightParen,
            "Expect ')' after for clauses".to_string(),
        )?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(BlockStmt(vec![
                body,
                Stmt::Expression(ExpressionStmt(increment)),
            ]));
        }

        let condition = if let Some(condition) = condition {
            condition
        } else {
            Expr::Literal(LiteralExpr(
                LiteralExprType::True,
                Token {
                    lexeme: "true".into(),
                    line: 0,
                    token_type: TokenType::True,
                },
            ))
        };

        body = Stmt::While(WhileStmt {
            condition,
            body: Box::new(body),
        });

        if let Some(initializer) = initializer {
            body = Stmt::Block(BlockStmt(vec![initializer, body]));
        }

        return Ok(body);
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.".to_string())?;

        return Ok(Stmt::Print(PrintStmt(value)));
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        let keyword = self.previous().cloned();

        let value = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after return value".to_string(),
        )?;

        return Ok(Stmt::Return(ReturnStmt {
            keyword: keyword.unwrap(),
            value,
        }));
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after expression.".to_string(),
        )?;

        return Ok(Stmt::Expression(ExpressionStmt(expr)));
    }

    fn function(&mut self, kind: LoxStr) -> Result<FunctionStmt> {
        let name = self.consume(&TokenType::Identifier, format!("Expect {kind} name."))?;

        self.consume(
            &TokenType::LeftParen,
            format!("Expect '(' after {kind} name."),
        )?;

        let mut parameters = vec![];

        if !self.check(&TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(self.error(
                        "Can't have more than 255 parameters".to_string(),
                        self.peek().unwrap().clone(),
                    ));
                }

                parameters.push(
                    self.consume(&TokenType::Identifier, "Expect parameter name".to_string())?,
                );

                if !self.match_any(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(
            &TokenType::RightParen,
            "Expect ')' after parameters".to_string(),
        )?;
        self.consume(
            &TokenType::LeftBrace,
            format!("Expect '{{' before {kind} body."),
        )?;

        let body = self.block()?;

        return Ok(FunctionStmt {
            name,
            params: parameters,
            body,
        });
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = vec![];

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if let Some(statement) = self.declaration() {
                statements.push(statement);
            }
        }

        self.consume(&TokenType::RightBrace, "Expect '}' after block".to_string())?;

        return Ok(statements);
    }

    fn expression(&mut self) -> Result<Expr> {
        return self.assignment();
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if self.match_any(&[TokenType::Equal]) {
            let equals = self.previous().cloned();
            let value = self.assignment()?;

            if let Expr::Variable(expr) = expr {
                let name = expr.0;
                return Ok(Expr::Assignment(AssignmentExpr {
                    name,
                    value: Box::new(value),
                }));
            }

            return Err(self.error(
                format!("[{}:{}] Invalid assignment target", file!(), line!()),
                equals.unwrap(),
            ));
        }

        return Ok(expr);
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.match_any(&[TokenType::Or]) {
            let operator = self.previous().unwrap().clone();
            let right = self.and()?;

            expr = Expr::Logical(LogicalExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.match_any(&[TokenType::And]) {
            let operator = self.previous().unwrap().clone();
            let right = self.equality()?;

            expr = Expr::Logical(LogicalExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        return Ok(expr);
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

        return self.call();
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        // loop {
        //     if self.match_any(&[TokenType::LeftParen]) {
        //         expr = self.finish_call(expr)?;
        //     } else {
        //         break;
        //     }
        // }

        while self.match_any(&[TokenType::LeftParen]) {
            expr = self.finish_call(expr)?;
        }

        return Ok(expr);
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = vec![];

        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(self.error(
                        "Can't have more than 255 arguments".to_string(),
                        self.peek().unwrap().clone(),
                    ));
                }

                arguments.push(self.expression()?);

                if !self.match_any(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(
            &TokenType::RightParen,
            "Expect ')' after arguments".to_string(),
        )?;

        return Ok(Expr::Call(CallExpr {
            callee: Box::new(callee),
            paren,
            arguments,
        }));
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
