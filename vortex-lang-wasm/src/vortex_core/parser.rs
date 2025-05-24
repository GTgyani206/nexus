// || shree ganesh ||
// Parser for the Vortex language

use super::ast::{Expr, Stmt};
use super::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                // Skip problematic tokens
                self.advance();
            }
        }
        statements
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.peek() {
            Token::Let => self.parse_let_statement(),
            Token::If => self.parse_if_statement(),
            Token::Branch => self.parse_branch_statement(),
            Token::Fallback => self.parse_fallback_statement(),
            Token::For => self.parse_for_statement(),
            Token::Parallel => self.parse_parallel_statement(),
            Token::Fn => self.parse_function_statement(false),
            Token::GPU => {
                self.advance(); // consume @gpu
                if self.peek() == &Token::Fn {
                    self.parse_function_statement(true)
                } else {
                    None
                }
            },
            Token::Return => self.parse_return_statement(),
            Token::Identifier(_) if self.peek_next() == Some(&Token::Lparen) => {
                Some(Stmt::ExprStmt(self.parse_expression()))
            },
            _ => {
                if self.is_expression_start() {
                    Some(Stmt::ExprStmt(self.parse_expression()))
                } else {
                    None
                }
            }
        }
    }

    fn parse_let_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'let'
        
        let mutable = if self.peek() == &Token::Mut {
            self.advance();
            true
        } else {
            false
        };

        let name = match self.advance() {
            Token::Identifier(id) => id,
            _ => return None,
        };

        let type_name = if self.match_token(&Token::Colon) {
            match self.advance() {
                Token::Identifier(t) => Some(t),
                _ => None,
            }
        } else {
            None
        };

        if !self.match_token(&Token::Equals) {
            return None;
        }

        let value = self.parse_expression();

        Some(Stmt::Let {
            name,
            type_name,
            value,
            mutable,
        })
    }

    fn parse_if_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'if'
        
        let condition = self.parse_expression();
        
        if !self.match_token(&Token::Colon) {
            return None;
        }

        let mut then_statements = Vec::new();
        while !matches!(self.peek(), Token::Then | Token::Else | Token::EOF | Token::Branch | Token::Fallback | Token::For | Token::Parallel | Token::Fn | Token::GPU | Token::Let) {
            if let Some(stmt) = self.parse_statement() {
                then_statements.push(stmt);
            } else {
                break;
            }
        }

        let then_branch = Box::new(Stmt::Block(then_statements));
        let mut else_branch = None;

        // Handle 'then' (else-if)
        if self.match_token(&Token::Then) {
            let else_if_condition = self.parse_expression();
            
            if !self.match_token(&Token::Colon) {
                return None;
            }

            let mut else_if_statements = Vec::new();
            while !matches!(self.peek(), Token::Else | Token::EOF | Token::Branch | Token::Fallback | Token::For | Token::Parallel | Token::Fn | Token::GPU | Token::Let) {
                if let Some(stmt) = self.parse_statement() {
                    else_if_statements.push(stmt);
                } else {
                    break;
                }
            }

            else_branch = Some(Box::new(Stmt::IfStmt {
                condition: else_if_condition,
                then_branch: Box::new(Stmt::Block(else_if_statements)),
                else_branch: None,
            }));
        }

        // Handle 'else'
        if self.match_token(&Token::Else) {
            if !self.match_token(&Token::Colon) {
                return None;
            }

            let mut else_statements = Vec::new();
            while !matches!(self.peek(), Token::EOF | Token::Branch | Token::Fallback | Token::For | Token::Parallel | Token::Fn | Token::GPU | Token::Let) {
                if let Some(stmt) = self.parse_statement() {
                    else_statements.push(stmt);
                } else {
                    break;
                }
            }

            if let Some(ref mut existing_else) = else_branch {
                if let Stmt::IfStmt { ref mut else_branch, .. } = **existing_else {
                    *else_branch = Some(Box::new(Stmt::Block(else_statements)));
                }
            } else {
                else_branch = Some(Box::new(Stmt::Block(else_statements)));
            }
        }

        Some(Stmt::IfStmt {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_branch_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'branch'
        
        let condition = self.parse_expression();
        
        if !self.match_token(&Token::FatArrow) {
            return None;
        }

        let body = if self.is_expression_start() {
            Box::new(Stmt::ExprStmt(self.parse_expression()))
        } else {
            Box::new(Stmt::Block(vec![]))
        };

        Some(Stmt::Branch { condition, body })
    }

    fn parse_fallback_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'fallback'
        
        if !self.match_token(&Token::FatArrow) {
            return None;
        }

        let body = if self.is_expression_start() {
            Box::new(Stmt::ExprStmt(self.parse_expression()))
        } else {
            Box::new(Stmt::Block(vec![]))
        };

        Some(Stmt::Fallback(body))
    }

    fn parse_for_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'for'
        
        let var = match self.advance() {
            Token::Identifier(id) => id,
            _ => return None,
        };

        if !self.match_token(&Token::In) {
            return None;
        }

        let range = self.parse_expression();

        if !self.match_token(&Token::Colon) {
            return None;
        }

        let mut statements = Vec::new();
        while !matches!(self.peek(), Token::EOF | Token::Branch | Token::Fallback | Token::For | Token::Parallel | Token::Fn | Token::GPU | Token::Let | Token::If) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                break;
            }
        }

        let body = Box::new(Stmt::Block(statements));

        Some(Stmt::For { var, range, body })
    }

    fn parse_parallel_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'parallel'
        
        let var = match self.advance() {
            Token::Identifier(id) => id,
            _ => return None,
        };

        if !self.match_token(&Token::In) {
            return None;
        }

        let range = self.parse_expression();

        if !self.match_token(&Token::Colon) {
            return None;
        }

        let mut statements = Vec::new();
        while !matches!(self.peek(), Token::EOF | Token::Branch | Token::Fallback | Token::For | Token::Parallel | Token::Fn | Token::GPU | Token::Let | Token::If) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                break;
            }
        }

        let body = Box::new(Stmt::Block(statements));

        Some(Stmt::Parallel { var, range, body })
    }

    fn parse_function_statement(&mut self, gpu: bool) -> Option<Stmt> {
        self.advance(); // consume 'fn'
        
        let name = match self.advance() {
            Token::Identifier(id) => id,
            _ => return None,
        };

        if !self.match_token(&Token::Lparen) {
            return None;
        }

        let mut params = Vec::new();
        if self.peek() != &Token::Rparen {
            loop {
                let param_name = match self.advance() {
                    Token::Identifier(id) => id,
                    _ => break,
                };

                let param_type = if self.match_token(&Token::Colon) {
                    match self.advance() {
                        Token::Identifier(t) => Some(t),
                        _ => None,
                    }
                } else {
                    None
                };

                params.push((param_name, param_type));

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        if !self.match_token(&Token::Rparen) {
            return None;
        }

        let return_type = if self.match_token(&Token::Arrow) {
            match self.advance() {
                Token::Identifier(t) => Some(t),
                _ => None,
            }
        } else {
            None
        };

        if !self.match_token(&Token::Colon) {
            return None;
        }

        let mut statements = Vec::new();
        while !matches!(self.peek(), Token::EOF | Token::Branch | Token::Fallback | Token::For | Token::Parallel | Token::Fn | Token::GPU | Token::Let | Token::If) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                break;
            }
        }

        let body = Box::new(Stmt::Block(statements));

        Some(Stmt::FunctionDef {
            name,
            params,
            return_type,
            body,
            gpu,
        })
    }

    fn parse_return_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'return'
        let value = self.parse_expression();
        Some(Stmt::Return(value))
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut expr = self.parse_term();

        while matches!(self.peek(), Token::GT | Token::GE | Token::LT | Token::LE | Token::EQ | Token::NE) {
            let op = self.advance();
            let right = self.parse_term();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_term(&mut self) -> Expr {
        let mut expr = self.parse_factor();

        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = self.advance();
            let right = self.parse_factor();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_factor(&mut self) -> Expr {
        let mut expr = self.parse_unary();

        while matches!(self.peek(), Token::Star | Token::Slash) {
            let op = self.advance();
            let right = self.parse_unary();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_unary(&mut self) -> Expr {
        if matches!(self.peek(), Token::Minus) {
            let op = self.advance();
            let expr = self.parse_unary();
            return Expr::Unary {
                op,
                expr: Box::new(expr),
            };
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Expr {
        let mut expr = self.parse_primary();

        while self.peek() == &Token::Lparen {
            self.advance(); // consume '('
            let mut args = Vec::new();

            if self.peek() != &Token::Rparen {
                args.push(self.parse_expression());
                while self.match_token(&Token::Comma) {
                    args.push(self.parse_expression());
                }
            }

            if !self.match_token(&Token::Rparen) {
                // Try to recover by breaking
                break;
            }

            expr = Expr::FunctionCall {
                callee: Box::new(expr),
                arguments: args,
            };
        }

        expr
    }

    fn parse_primary(&mut self) -> Expr {
        match self.advance() {
            Token::Number(n) => {
                // Check for range syntax
                if matches!(self.peek(), Token::Range | Token::Dot) {
                    self.parse_range_from_start(Expr::Number(n))
                } else {
                    Expr::Number(n)
                }
            },
            Token::Floating(f) => {
                // Check for range syntax
                if matches!(self.peek(), Token::Range | Token::Dot) {
                    self.parse_range_from_start(Expr::Floating(f))
                } else {
                    Expr::Floating(f)
                }
            },
            Token::Boolean(b) => Expr::Boolean(b),
            Token::String(s) => Expr::String(s),
            Token::Identifier(id) => {
                // Check for range syntax or function calls
                if matches!(self.peek(), Token::Range | Token::Dot) {
                    self.parse_range_from_start(Expr::Ident(id))
                } else {
                    Expr::Ident(id)
                }
            },
            Token::Range => {
                // Handle range(start, end) function
                if self.match_token(&Token::Lparen) {
                    let start = self.parse_expression();
                    if self.match_token(&Token::Comma) {
                        let end = self.parse_expression();
                        if self.match_token(&Token::Rparen) {
                            Expr::Range {
                                start: Box::new(start),
                                end: Box::new(end),
                            }
                        } else {
                            Expr::Ident("error".to_string())
                        }
                    } else {
                        Expr::Ident("error".to_string())
                    }
                } else {
                    Expr::Ident("range".to_string())
                }
            },
            Token::Lparen => {
                let expr = self.parse_expression();
                if self.match_token(&Token::Rparen) {
                    Expr::Grouping(Box::new(expr))
                } else {
                    expr
                }
            },
            _ => Expr::Ident("error".to_string()),
        }
    }

    fn parse_range_from_start(&mut self, start: Expr) -> Expr {
        if self.match_token(&Token::Range) {
            // Handle .. operator
            let end = self.parse_expression();
            Expr::Range {
                start: Box::new(start),
                end: Box::new(end),
            }
        } else if self.match_token(&Token::Dot) {
            // Handle potential .. operator
            if self.match_token(&Token::Dot) {
                let end = self.parse_expression();
                Expr::Range {
                    start: Box::new(start),
                    end: Box::new(end),
                }
            } else {
                // Just a single dot - not a range
                start
            }
        } else {
            start
        }
    }

    fn is_expression_start(&self) -> bool {
        matches!(
            self.peek(),
            Token::Number(_) | Token::Floating(_) | Token::Boolean(_) | 
            Token::String(_) | Token::Identifier(_) | Token::Lparen | 
            Token::Minus | Token::Range
        )
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::EOF)
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap_or(&Token::EOF).clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek() == &Token::EOF
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.peek()) == std::mem::discriminant(token)
        }
    }
}