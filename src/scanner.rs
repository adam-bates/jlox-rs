use crate::{lox, token::Token, token_type::TokenType, string::LoxStr};

use std::{collections::HashMap, iter::Iterator};

use lazy_static::lazy_static;

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut keywords = HashMap::new();
        keywords.insert(String::from("and"), TokenType::And);
        keywords.insert(String::from("class"), TokenType::Class);
        keywords.insert(String::from("else"), TokenType::Else);
        keywords.insert(String::from("false"), TokenType::False);
        keywords.insert(String::from("for"), TokenType::For);
        keywords.insert(String::from("fun"), TokenType::Fun);
        keywords.insert(String::from("if"), TokenType::If);
        keywords.insert(String::from("nil"), TokenType::Nil);
        keywords.insert(String::from("or"), TokenType::Or);
        keywords.insert(String::from("print"), TokenType::Print);
        keywords.insert(String::from("return"), TokenType::Return);
        keywords.insert(String::from("super"), TokenType::Super);
        keywords.insert(String::from("this"), TokenType::This);
        keywords.insert(String::from("true"), TokenType::True);
        keywords.insert(String::from("var"), TokenType::Var);
        keywords.insert(String::from("while"), TokenType::While);
        keywords
    };
}

pub struct Scanner {
    source: LoxStr,
    source_chars: Vec<char>,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: LoxStr) -> Self {
        return Self {
            source_chars: source.chars().collect(),
            source,
            tokens: Vec::new(),

            start: 0,
            current: 0,
            line: 1,
        };
    }

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: "".into(),
            line: self.line,
        });

        return self.tokens;
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        let token_type = match c {
            '(' => Some(TokenType::LeftParen),
            ')' => Some(TokenType::RightParen),
            '{' => Some(TokenType::LeftBrace),
            '}' => Some(TokenType::RightBrace),
            ',' => Some(TokenType::Comma),
            '.' => Some(TokenType::Dot),
            '-' => Some(TokenType::Minus),
            '+' => Some(TokenType::Plus),
            ';' => Some(TokenType::Semicolon),
            '*' => Some(TokenType::Star),

            '!' => Some(if self.match_next('=') {
                TokenType::BangEqual
            } else {
                TokenType::Bang
            }),

            '=' => Some(if self.match_next('=') {
                TokenType::EqualEqual
            } else {
                TokenType::Equal
            }),

            '<' => Some(if self.match_next('=') {
                TokenType::LessEqual
            } else {
                TokenType::Less
            }),

            '>' => Some(if self.match_next('=') {
                TokenType::GreaterEqual
            } else {
                TokenType::Greater
            }),

            '/' => {
                if self.match_next('/') {
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        self.advance();
                    }

                    None
                } else {
                    Some(TokenType::Slash)
                }
            }

            // Ignore whitespace
            ' ' | '\r' | '\t' => None,

            '\n' => {
                self.line += 1;
                None
            }

            '"' => Some(self.string()),

            c if self.is_digit(c) => Some(self.number()),

            c if self.is_alpha(c) => Some(self.identifier()),

            _ => {
                lox::error(self.line, "Unexpected character.");
                None
            }
        };

        if let Some(token_type) = token_type {
            self.add_token(token_type);
        }
    }

    fn string(&mut self) -> TokenType {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            lox::error(self.line, "Unterminated string.");
        }

        // The closing ".
        self.advance();

        // Trim the surrounding quotes
        let value = self.source[self.start + 1..self.current - 1].to_string();
        return TokenType::String(value.into());
    }

    fn number(&mut self) -> TokenType {
        while let Some(peek) = self.peek() {
            if !self.is_digit(peek) {
                break;
            }

            self.advance();
        }

        // Look for a fractional park.
        if self.peek() == Some('.') {
            if let Some(next) = self.peek_next() {
                if self.is_digit(next) {
                    // Consume the "."
                    self.advance();

                    while let Some(peek) = self.peek() {
                        if !self.is_digit(peek) {
                            break;
                        }

                        self.advance();
                    }
                }
            }
        }

        return TokenType::Number(
            self.source[self.start..self.current]
                .parse::<f64>()
                .unwrap(),
        );
    }

    fn identifier(&mut self) -> TokenType {
        while let Some(peek) = self.peek() {
            if !self.is_alpha_numeric(peek) {
                break;
            }

            self.advance();
        }

        let text = &self.source[self.start..self.current];
        return KEYWORDS.get(text).cloned().unwrap_or(TokenType::Identifier);
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source_chars[self.current] != expected {
            return false;
        }

        self.current += 1;

        return true;
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }

        return Some(self.source_chars[self.current]);
    }

    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            return None;
        }

        return Some(self.source_chars[self.current + 1]);
    }

    fn is_alpha(&self, c: char) -> bool {
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
    }

    fn is_digit(&self, c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        return self.is_alpha(c) || self.is_digit(c);
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    fn advance(&mut self) -> char {
        let c = self.source_chars[self.current];

        self.current += 1;

        return c;
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.source[self.start..self.current];

        self.tokens.push(Token {
            token_type,
            lexeme: text.to_string().into(),
            line: self.line,
        });
    }
}
