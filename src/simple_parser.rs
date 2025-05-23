// || shree ganesh ||
// This file contains a simplified parser for the Vortex language.

use crate::token::Token;
use crate::ast::{Stmt, Expr};

pub struct SimpleParser {
    tokens: Vec<Token>,
    current: usize,
}

impl SimpleParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        SimpleParser {
            tokens,
            current: 0,
        }
    }
    
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }
        
        statements
    }
    
    fn declaration(&mut self) -> Option<Stmt> {
        if self.match_token(&Token::Let) {
            return self.let_declaration();
        }
        
        self.statement()
    }
    
    fn let_declaration(&mut self) -> Option<Stmt> {
        let mutable = self.match_token(&Token::Mut);
        
        if let Token::Identifier(name) = self.advance() {
            // Type annotation (optional)
            let type_name = if self.match_token(&Token::Colon) {
                if let Token::Identifier(type_name) = self.advance() {
                    Some(type_name)
                } else {
                    None
                }
            } else {
                None
            };
            
            // Initialize with a value
            if self.match_token(&Token::Equals) {
                let value = self.expression();
                
                return Some(Stmt::Let {
                    name,
                    type_name,
                    value,
                    mutable,
                });
            }
        }
        
        None
    }
    
    fn statement(&mut self) -> Option<Stmt> {
        if self.match_token(&Token::If) {
            return self.if_statement();
        }
        
        // Expression statement
        let expr = self.expression();
        Some(Stmt::ExprStmt(expr))
    }
    
    fn if_statement(&mut self) -> Option<Stmt> {
        let condition = self.expression();
        
        if !self.match_token(&Token::Colon) {
            return None;
        }
        
        let then_branch = Box::new(self.block_statement());
        
        let else_branch = if self.match_token(&Token::Else) {
            if !self.match_token(&Token::Colon) {
                return None;
            }
            
            Some(Box::new(self.block_statement()))
        } else {
            None
        };
        
        Some(Stmt::IfStmt {
            condition,
            then_branch,
            else_branch,
        })
    }
    
    fn block_statement(&mut self) -> Stmt {
        let mut statements = Vec::new();
        
        while !self.is_at_end() && !self.check(&Token::Else) && !self.check(&Token::EOF) {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            } else {
                break;
            }
        }
        
        Stmt::Block(statements)
    }
    
    fn expression(&mut self) -> Expr {
        self.assignment()
    }
    
    fn assignment(&mut self) -> Expr {
        let expr = self.equality();
        
        if self.match_token(&Token::Equals) {
            if let Expr::Ident(name) = expr {
                let value = self.assignment();
                return Expr::Assignment {
                    name,
                    value: Box::new(value),
                };
            }
        }
        
        expr
    }
    
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        
        while self.match_any(&[Token::EQ, Token::NE]) {
            let operator = self.previous();
            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }
        
        expr
    }
    
    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        
        while self.match_any(&[Token::GT, Token::GE, Token::LT, Token::LE]) {
            let operator = self.previous();
            let right = self.term();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }
        
        expr
    }
    
    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        
        while self.match_any(&[Token::Plus, Token::Minus]) {
            let operator = self.previous();
            let right = self.factor();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }
        
        expr
    }
    
    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        
        while self.match_any(&[Token::Star, Token::Slash]) {
            let operator = self.previous();
            let right = self.unary();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }
        
        expr
    }
    
    fn unary(&mut self) -> Expr {
        if self.match_any(&[Token::Minus]) {
            let operator = self.previous();
            let right = self.unary();
            return Expr::Unary {
                op: operator,
                expr: Box::new(right),
            };
        }
        
        self.call()
    }
    
    fn call(&mut self) -> Expr {
        let mut expr = self.primary();
        
        loop {
            if self.match_token(&Token::Lparen) {
                expr = self.finish_call(expr);
            } else {
                break;
            }
        }
        
        expr
    }
    
    fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut arguments = Vec::new();
        
        if !self.check(&Token::Rparen) {
            loop {
                arguments.push(self.expression());
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(Token::Rparen, "Expected ')' after arguments.");
        
        Expr::FunctionCall {
            callee: Box::new(callee),
            arguments,
        }
    }
    
    fn primary(&mut self) -> Expr {
        if self.match_token(&Token::Number(0)) {
            if let Token::Number(value) = self.previous() {
                return Expr::Number(value);
            }
        }
        
        if self.match_token(&Token::String(String::new())) {
            if let Token::String(value) = self.previous() {
                return Expr::String(value);
            }
        }
        
        if self.match_token(&Token::Boolean(false)) {
            if let Token::Boolean(value) = self.previous() {
                return Expr::Boolean(value);
            }
        }
        
        if self.match_token(&Token::Identifier(String::new())) {
            if let Token::Identifier(name) = self.previous() {
                return Expr::Ident(name);
            }
        }
        
        if self.match_token(&Token::Lparen) {
            let expr = self.expression();
            self.consume(Token::Rparen, "Expected ')' after expression.");
            return Expr::Grouping(Box::new(expr));
        }
        
        // Default error case
        Expr::Ident(String::from("error"))
    }
    
    // Helper methods
    
    fn match_token(&mut self, token_type: &Token) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn match_any(&mut self, token_types: &[Token]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }
    
    fn consume(&mut self, token_type: Token, message: &str) -> Token {
        if self.check(&token_type) {
            return self.advance();
        }
        
        panic!("{} Found {:?}", message, self.peek());
    }
    
    fn check(&self, token_type: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        
        match (self.peek(), token_type) {
            (Token::Number(_), Token::Number(_)) => true,
            (Token::String(_), Token::String(_)) => true,
            (Token::Boolean(_), Token::Boolean(_)) => true,
            (Token::Identifier(_), Token::Identifier(_)) => true,
            _ => self.peek() == token_type,
        }
    }
    
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek() == &Token::EOF
    }
    
    fn peek(&self) -> &Token {
        if self.current < self.tokens.len() {
            &self.tokens[self.current]
        } else {
            // If we're at the end, return EOF
            &Token::EOF
        }
    }
    
    fn previous(&self) -> Token {
        if self.current > 0 {
            self.tokens[self.current - 1].clone()
        } else {
            // Return EOF if we're at the beginning
            Token::EOF
        }
    }
}