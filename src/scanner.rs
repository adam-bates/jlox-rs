use crate::{
    lox,
    token::{Literal, Token},
    token_type::TokenType,
};

use std::{cell::RefCell, rc::Rc};

pub struct Scanner {
    source: String,
    tokens: Rc<RefCell<Vec<Token>>>,

    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        return Self {
            source,
            tokens: Rc::new(RefCell::new(Vec::new())),

            start: 0,
            current: 0,
            line: 1,
        };
    }

    pub fn scan_tokens(&mut self) -> Rc<RefCell<Vec<Token>>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.borrow_mut().push(Token {
            token_type: TokenType::EOF,
            lexeme: "".to_string(),
            literal: None,
            line: self.line,
        });

        return Rc::clone(&self.tokens);
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),

            _ => lox::error(self.line, "Unexpected character."),
        }
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current..].chars().next().unwrap();

        self.current += 1;

        return c;
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token2(token_type, None);
    }

    fn add_token2(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = &self.source[self.start..self.current];

        self.tokens.borrow_mut().push(Token {
            token_type,
            lexeme: text.to_string(),
            literal,
            line: self.line,
        });
    }
}
