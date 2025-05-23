// || shree ganesh ||
// Parser for the Vortex language

use crate::ast::{Expr, Stmt};
use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.parse_stmt() {
                println!("Successfully parsed statement: {:?}", stmt);
                stmts.push(stmt);
            } else {
                // Skip this token and try to recover
                println!("Failed to parse statement at token: {:?}", self.peek());
                self.advance();
            }
        }
        stmts
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::EOF)
    }

    fn peek_next(&self) -> Option<&Token> {
        if self.current + 1 < self.tokens.len() {
            Some(&self.tokens[self.current + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if !self.is_at_end() {
            self.current += 1;
        }
        tok
    }

    fn check(&self, token: &Token) -> bool {
        self.peek() == token
    }

    fn expect(&mut self, expected: Token) -> bool {
        if self.check(&expected) {
            self.advance();
            true
        } else {
            println!("Expected {:?}, found {:?}", expected, self.peek());
            false
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek() == &Token::EOF
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        println!("Parsing statement, current token: {:?}", self.peek());

        // Skip any tokens that are clearly just syntax errors
        if matches!(
            self.peek(),
            Token::Colon | Token::Comma | Token::Rparen | Token::Dot
        ) {
            println!(
                "Skipping unexpected token in statement context: {:?}",
                self.peek()
            );
            self.advance();
            return self.parse_stmt();
        }

        match self.peek() {
            Token::Let => Some(self.parse_let()),
            Token::If => self.parse_if_stmt(),
            Token::Branch => self.parse_branch(),
            Token::Fallback => self.parse_fallback(),
            Token::For => self.parse_loop(false),
            Token::Parallel => self.parse_loop(true),
            Token::GPU => {
                self.advance(); // consume GPU
                if self.expect(Token::Fn) {
                    self.parse_function_def(true)
                } else {
                    println!("Expected `fn` after `@gpu`");
                    None
                }
            }
            Token::Fn => self.parse_function_def(false),
            Token::Return => {
                self.advance();
                let expr = self.parse_expr();
                Some(Stmt::Return(expr))
            }
            // If we see a string or identifier as the first token, it's likely a function call
            Token::Identifier(_) if self.peek_next() == Some(&Token::Lparen) => {
                Some(self.parse_expr_stmt())
            }
            Token::EOF => None, // End of input
            _ => {
                // Try to parse as expression statement
                let expr_stmt = self.parse_expr_stmt();
                Some(expr_stmt)
            }
        }
    }

    fn parse_let(&mut self) -> Stmt {
        self.advance(); // 'let'
        let mutable = self.match_token(&Token::Mut);

        // Check if we got a valid identifier
        let name = match self.peek() {
            Token::Identifier(_) => match self.advance() {
                Token::Identifier(n) => n,
                _ => unreachable!(),
            },
            other => {
                println!("Expected identifier after let, found {:?}", other);
                "error".to_string()
            }
        };

        // Parse type annotation if present
        let type_name = if self.match_token(&Token::Colon) {
            match self.peek() {
                Token::Identifier(_) => match self.advance() {
                    Token::Identifier(t) => Some(t),
                    _ => unreachable!(),
                },
                other => {
                    println!("Expected type name after colon, found {:?}", other);
                    Some("error".to_string())
                }
            }
        } else {
            None
        };

        // Expect equals sign
        if !self.expect(Token::Equals) {
            println!("Expected '=' after let declaration");
        }

        // Parse the initializer expression
        let value = self.parse_expr();

        Stmt::Let {
            name,
            type_name,
            value,
            mutable,
        }
    }

    fn parse_expr_stmt(&mut self) -> Stmt {
        Stmt::ExprStmt(self.parse_expr())
    }

    fn parse_expr(&mut self) -> Expr {
        // Check for tokens that should never start an expression
        match self.peek() {
            Token::Let
            | Token::Mut
            | Token::Then
            | Token::Else
            | Token::Branch
            | Token::Fallback
            | Token::For
            | Token::Parallel
            | Token::Fn
            | Token::Colon => {
                println!(
                    "Warning: Found {:?} in expression context, which is invalid",
                    self.peek()
                );
                self.advance(); // Skip this invalid token
                return Expr::Ident("error".to_string()); // Return a placeholder
            }
            _ => {
                let result = self.parse_unary();
                // Debug the parsed expression
                println!("Parsed expression: {:?}", result);
                result
            }
        }
    }

    fn parse_unary(&mut self) -> Expr {
        match self.peek() {
            Token::Minus => {
                let op = self.advance();
                let expr = self.parse_unary();
                Expr::Unary {
                    op,
                    expr: Box::new(expr),
                }
            }
            _ => self.parse_prec_expr(0),
        }
    }

    fn parse_prec_expr(&mut self, min_prec: u8) -> Expr {
        let mut left = self.parse_primary();

        // Special handling for expression errors - don't try to continue parsing binary ops
        if let Expr::Ident(id) = &left {
            if id == "error" {
                return left;
            }
        }

        while Self::get_precedence(self.peek()) >= min_prec {
            println!("Parsing binary expression with operator: {:?}", self.peek());

            let op = self.advance();

            // Skip any whitespace tokens after the operator
            self.skip_whitespace_tokens();

            // Check for invalid tokens that shouldn't be expression starters
            match self.peek() {
                Token::EOF => {
                    println!("Warning: Unexpected end of input after operator");
                    break;
                }
                Token::Let
                | Token::Mut
                | Token::Else
                | Token::Then
                | Token::Branch
                | Token::Fallback
                | Token::For
                | Token::Parallel
                | Token::Fn
                | Token::Colon => {
                    println!(
                        "Warning: Found {:?} after operator, which is invalid",
                        self.peek()
                    );
                    // Abort the binary expression parsing
                    return left;
                }
                _ => {
                    // Try to parse the right operand
                    let right = self.parse_primary();

                    // If we got an error parsing the right side, abort the binary expression
                    if let Expr::Ident(id) = &right {
                        if id == "error" {
                            return left;
                        }
                    }

                    // Check for higher precedence operators
                    let next_prec = Self::get_precedence(self.peek());
                    let op_prec = Self::get_precedence(&op);

                    let right_expr = if next_prec > op_prec {
                        // If the next operator has higher precedence, parse that first
                        self.parse_prec_expr(op_prec + 1)
                    } else {
                        right
                    };

                    left = Expr::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right_expr),
                    };
                }
            }
        }

        left
    }

    fn get_precedence(tok: &Token) -> u8 {
        match tok {
            Token::EQ | Token::NE => 1,
            Token::LT | Token::LE | Token::GT | Token::GE => 2,
            Token::Plus | Token::Minus => 3,
            Token::Star | Token::Slash => 4,
            Token::Range => 5,
            Token::Dot => 6,
            _ => 0,
        }
    }

    fn parse_primary(&mut self) -> Expr {
        println!("Parsing primary, current token: {:?}", self.peek());

        // Skip any whitespace tokens
        self.skip_whitespace_tokens();

        if self.is_at_end() {
            println!("Warning: Unexpected end of input while parsing primary expression");
            return Expr::Ident("error".to_string());
        }

        // Handle special tokens that should not be part of expressions
        match self.peek() {
            Token::Let
            | Token::Mut
            | Token::Colon
            | Token::Then
            | Token::Else
            | Token::Branch
            | Token::Fallback
            | Token::For
            | Token::Parallel
            | Token::Fn
            | Token::Return => {
                println!(
                    "Warning: Found {:?} in expression context which is invalid",
                    self.peek()
                );
                self.advance(); // Skip this token
                return Expr::Ident("error".to_string());
            }
            _ => {}
        }

        match self.advance() {
            Token::Number(n) => {
                println!("Parsed number literal: {}", n);
                // Look ahead for range syntax: 0..10
                if self.peek() == &Token::Dot || self.peek() == &Token::Range {
                    return self.parse_range_after_start(Expr::Number(n));
                }
                Expr::Number(n)
            }
            Token::Floating(f) => {
                println!("Parsed floating-point literal: {}", f);
                // Look ahead for range syntax with floats
                if self.peek() == &Token::Dot || self.peek() == &Token::Range {
                    return self.parse_range_after_start(Expr::Floating(f));
                }
                Expr::Floating(f)
            }
            Token::Boolean(b) => {
                println!("Parsed boolean literal: {}", b);
                Expr::Boolean(b)
            }
            Token::String(s) => {
                println!("Parsed string literal: \"{}\"", s);
                Expr::String(s)
            }
            Token::Range => {
                println!("Parsing range function call");
                // Handle the "range" function call syntax: range(0, 3)
                if self.peek() == &Token::Lparen {
                    self.advance(); // consume '('
                    let start = self.parse_expr();

                    if self.peek() == &Token::Comma {
                        self.advance(); // consume ','
                        let end = self.parse_expr();

                        if self.expect(Token::Rparen) {
                            println!("Completed range expression");
                            Expr::Range {
                                start: Box::new(start),
                                end: Box::new(end),
                            }
                        } else {
                            println!("Expected closing parenthesis after range arguments");
                            Expr::Ident("error".to_string())
                        }
                    } else {
                        println!("Expected comma in range function");
                        Expr::Ident("error".to_string())
                    }
                } else {
                    println!("Expected opening parenthesis after range");
                    Expr::Ident("error".to_string())
                }
            }
            Token::Identifier(id) => {
                println!("Parsing identifier: {}", id);

                // Special handling for specific identifiers
                match id.as_str() {
                    "print" => {
                        // Handle print function call
                        if self.peek() == &Token::Lparen {
                            println!("Parsing print function call");
                            self.advance(); // consume '('
                            let mut args = Vec::new();

                            if self.peek() != &Token::Rparen {
                                // Parse first argument
                                args.push(self.parse_expr());

                                // Parse additional arguments
                                while self.match_token(&Token::Comma) {
                                    args.push(self.parse_expr());
                                }
                            }

                            if self.expect(Token::Rparen) {
                                println!(
                                    "Completed print function call with {} arguments",
                                    args.len()
                                );
                                Expr::FunctionCall {
                                    callee: Box::new(Expr::Ident(id)),
                                    arguments: args,
                                }
                            } else {
                                println!("Expected closing parenthesis after function arguments");
                                // Even if we're missing the closing paren, create the call
                                Expr::FunctionCall {
                                    callee: Box::new(Expr::Ident(id)),
                                    arguments: args,
                                }
                            }
                        } else {
                            Expr::Ident(id)
                        }
                    }
                    "range" => {
                        // Special handling for range function
                        if self.peek() == &Token::Lparen {
                            println!("Parsing range function call");
                            self.advance(); // consume '('
                            let start = self.parse_expr();

                            if !self.expect(Token::Comma) {
                                println!("Expected comma in range function");
                                return Expr::Ident("error".to_string());
                            }

                            let end = self.parse_expr();

                            if !self.expect(Token::Rparen) {
                                println!("Expected closing parenthesis after range arguments");
                                return Expr::Ident("error".to_string());
                            }

                            println!("Completed range expression");
                            Expr::Range {
                                start: Box::new(start),
                                end: Box::new(end),
                            }
                        } else {
                            Expr::Ident(id)
                        }
                    }
                    _ => {
                        // Handle other identifiers
                        if self.peek() == &Token::Lparen {
                            println!("Parsing function call to {}", id);
                            self.advance(); // consume '('
                            let mut args = Vec::new();

                            if self.peek() != &Token::Rparen {
                                // Parse first argument
                                args.push(self.parse_expr());

                                // Parse additional arguments
                                while self.match_token(&Token::Comma) {
                                    args.push(self.parse_expr());
                                }
                            }

                            if self.expect(Token::Rparen) {
                                println!(
                                    "Completed function call to {} with {} arguments",
                                    id,
                                    args.len()
                                );
                                Expr::FunctionCall {
                                    callee: Box::new(Expr::Ident(id)),
                                    arguments: args,
                                }
                            } else {
                                println!("Expected closing parenthesis after function arguments");
                                // Create the function call anyway to continue parsing
                                Expr::FunctionCall {
                                    callee: Box::new(Expr::Ident(id)),
                                    arguments: args,
                                }
                            }
                        } else if self.peek() == &Token::Dot || self.peek() == &Token::Range {
                            // Handle range expression: i..10
                            println!("Parsing range expression starting with identifier");
                            return self.parse_range_after_start(Expr::Ident(id));
                        } else {
                            Expr::Ident(id)
                        }
                    }
                }
            }
            Token::Lparen => {
                println!("Parsing grouped expression");
                let expr = self.parse_expr();
                if self.expect(Token::Rparen) {
                    println!("Completed grouped expression");
                    Expr::Grouping(Box::new(expr))
                } else {
                    println!("Expected closing parenthesis after grouped expression");
                    Expr::Grouping(Box::new(expr)) // Try to recover anyway
                }
            }
            Token::Return => {
                println!("Parsing return expression");
                let value = self.parse_expr();
                println!("Completed return expression");
                Expr::Return(Box::new(value))
            }
            token => {
                println!("Unexpected token in expression: {:?}", token);
                Expr::Ident("error".to_string())
            }
        }
    }

    fn parse_range_after_start(&mut self, start_expr: Expr) -> Expr {
        println!("Parsing range expression after start");

        // Check if we have a range token
        if self.peek() == &Token::Range {
            // Range token (..) already lexed as one token
            self.advance(); // consume the Range token
        } else if self.peek() == &Token::Dot {
            // Handle the case of two consecutive dots (..)
            self.advance(); // consume first dot
            if self.peek() == &Token::Dot {
                self.advance(); // consume second dot
            } else {
                // If we only see one dot, this is a property access, not a range
                println!("Warning: Unexpected single dot, expected .. for range");
                // Return the original expression to avoid errors
                return start_expr;
            }
        } else {
            // Neither Range nor Dot token found
            println!("Expected range operator (..) after start expression");
            return start_expr;
        }

        // Now parse the end of the range
        let end_expr = self.parse_expr();

        Expr::Range {
            start: Box::new(start_expr),
            end: Box::new(end_expr),
        }
    }

    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        self.advance(); // Consume "if"

        println!("Parsing if condition");
        // Parse the condition
        let condition = self.parse_expr();

        // Expect colon after condition
        if !self.expect(Token::Colon) {
            println!("Expected colon after if condition");
            return None;
        }

        println!("Parsing if body");
        // Parse the body of the if statement
        let then_branch = Box::new(self.parse_block());

        // Check for else-if ("then") or else branches
        let mut else_branch = None;

        // Handle "then" clause (which is like else-if)
        if self.match_token(&Token::Then) {
            println!("Found 'then' branch (else-if)");
            // This is an else-if branch
            // Parse the condition for the else-if
            let else_if_condition = self.parse_expr();

            if !self.expect(Token::Colon) {
                println!("Expected colon after 'then' condition");
                return None;
            }

            println!("Parsing 'then' branch body");
            // Parse the body of the else-if
            let else_if_body = Box::new(self.parse_block());

            // Create nested if for the else-if
            else_branch = Some(Box::new(Stmt::IfStmt {
                condition: else_if_condition,
                then_branch: else_if_body,
                else_branch: None, // Will be updated if there's an else
            }));
        }

        // Handle "else" clause
        if self.match_token(&Token::Else) {
            println!("Found 'else' branch");
            if !self.expect(Token::Colon) {
                println!("Expected colon after 'else'");
                return None;
            }

            println!("Parsing 'else' branch body");
            let else_body = Box::new(self.parse_block());

            if let Some(ref mut branch) = else_branch {
                println!("Adding 'else' to 'then' branch");
                // We had a "then" clause, so add the else to that
                if let Stmt::IfStmt {
                    ref mut else_branch,
                    ..
                } = **branch
                {
                    *else_branch = Some(Box::new(*else_body));
                }
            } else {
                println!("Adding 'else' directly to 'if'");
                // Direct else for the if
                else_branch = Some(Box::new(*else_body));
            }
        }

        println!("Completed if-then-else statement");
        // Return the complete if statement
        Some(Stmt::IfStmt {
            condition,
            then_branch,
            else_branch,
        })
    }

    // Keep the old parse_if for backward compatibility, just delegate to the new implementation
    fn parse_if(&mut self) -> Option<Stmt> {
        self.parse_if_stmt()
    }

    fn parse_branch(&mut self) -> Option<Stmt> {
        self.advance(); // branch
        println!("Parsing branch condition");
        let condition = self.parse_expr();

        if !self.expect(Token::FatArrow) {
            println!("Expected '=>' after branch condition");
            return None;
        }

        println!("Parsing branch body after '=>'");

        // First try to parse a single expression statement (like a function call)
        if matches!(self.peek(), &Token::Identifier(_)) && self.peek_next() == Some(&Token::Lparen)
        {
            // This is likely a function call
            let expr = self.parse_expr();
            return Some(Stmt::Branch {
                condition,
                body: Box::new(Stmt::ExprStmt(expr)),
            });
        }

        // Otherwise, parse a statement or block
        let stmt = self.parse_stmt();
        if let Some(stmt) = stmt {
            Some(Stmt::Branch {
                condition,
                body: Box::new(stmt),
            })
        } else {
            println!("Expected statement after '=>'");
            None
        }
    }

    fn parse_fallback(&mut self) -> Option<Stmt> {
        self.advance(); // fallback

        if !self.expect(Token::FatArrow) {
            println!("Expected '=>' after 'fallback'");
            return None;
        }

        println!("Parsing fallback body after '=>'");

        // First try to parse a single expression statement (like a function call)
        if matches!(self.peek(), &Token::Identifier(_)) && self.peek_next() == Some(&Token::Lparen)
        {
            // This is likely a function call
            let expr = self.parse_expr();
            return Some(Stmt::Fallback(Box::new(Stmt::ExprStmt(expr))));
        }

        // Otherwise, parse a statement or block
        let stmt = self.parse_stmt();
        if let Some(stmt) = stmt {
            Some(Stmt::Fallback(Box::new(stmt)))
        } else {
            println!("Expected statement after '=>'");
            None
        }
    }

    fn parse_loop(&mut self, parallel: bool) -> Option<Stmt> {
        self.advance(); // for or parallel
        let var = match self.advance() {
            Token::Identifier(id) => id,
            other => {
                println!("Expected identifier in loop, found {:?}", other);
                return None;
            }
        };

        if !self.expect(Token::In) {
            println!("Expected 'in' after loop variable");
            return None;
        }

        println!("Parsing loop range after 'in'");

        // Handle both range expression formats:
        // 1. range(0, 10)
        if self.peek() == &Token::Range {
            // Handle the range function
            self.advance(); // consume 'range'

            if !self.expect(Token::Lparen) {
                println!("Expected '(' after range");
                return None;
            }

            let start = self.parse_expr();

            if !self.expect(Token::Comma) {
                println!("Expected ',' in range function");
                return None;
            }

            let end = self.parse_expr();

            if !self.expect(Token::Rparen) {
                println!("Expected ')' after range arguments");
                return None;
            }

            // Create a range expression
            let range = Expr::Range {
                start: Box::new(start),
                end: Box::new(end),
            };

            if !self.expect(Token::Colon) {
                println!("Expected ':' after range");
                return None;
            }

            println!("Parsing loop body");
            let body = Box::new(self.parse_block());

            if parallel {
                println!("Created parallel loop statement");
                Some(Stmt::Parallel { var, range, body })
            } else {
                println!("Created for loop statement");
                Some(Stmt::For { var, range, body })
            }
        }
        // 2. Direct number
        else if matches!(self.peek(), &Token::Number(_) | &Token::Floating(_)) {
            // Consume the number
            let start_expr = match self.advance() {
                Token::Number(n) => Expr::Number(n),
                Token::Floating(f) => Expr::Floating(f),
                _ => unreachable!(),
            };

            // Check for range operator (..)
            if self.check(&Token::Range) || self.check(&Token::Dot) {
                let range = self.parse_range_after_start(start_expr);

                if !self.expect(Token::Colon) {
                    println!("Expected ':' after range expression in loop");
                    return None;
                }

                println!("Parsing loop body");
                let body = Box::new(self.parse_block());

                if parallel {
                    println!("Created parallel loop statement with range");
                    Some(Stmt::Parallel { var, range, body })
                } else {
                    println!("Created for loop statement with range");
                    Some(Stmt::For { var, range, body })
                }
            } else {
                // Just a number without a range operator
                if !self.expect(Token::Colon) {
                    println!("Expected ':' after loop bound");
                    return None;
                }

                // Treat as 0..n range
                let range = Expr::Range {
                    start: Box::new(Expr::Number(0)),
                    end: Box::new(start_expr),
                };

                println!("Parsing loop body");
                let body = Box::new(self.parse_block());

                if parallel {
                    println!("Created parallel loop statement with implicit range");
                    Some(Stmt::Parallel { var, range, body })
                } else {
                    println!("Created for loop statement with implicit range");
                    Some(Stmt::For { var, range, body })
                }
            }
        }
        // 3. Any other expression
        else {
            // Parse regular expression which might be a range (x..y)
            let range = self.parse_expr();

            // Expect a colon after the range expression
            if !self.expect(Token::Colon) {
                println!("Expected ':' after range expression in loop");
                return None;
            }

            println!("Parsing loop body");
            let body = Box::new(self.parse_block());

            if parallel {
                println!("Created parallel loop statement with expression");
                Some(Stmt::Parallel { var, range, body })
            } else {
                println!("Created for loop statement with expression");
                Some(Stmt::For { var, range, body })
            }
        }
    }

    fn parse_block(&mut self) -> Stmt {
        let mut stmts = Vec::new();

        // Skip any initial whitespace tokens
        self.skip_whitespace_tokens();

        // Define tokens that terminate a block (cause us to exit this block's parsing)
        let block_terminators = [
            Token::Else,
            Token::Then,
            Token::EOF,
            Token::Branch,
            Token::Fallback,
        ];

        println!("Parsing block of statements");

        // Continue parsing statements until we find a block terminator
        while !block_terminators.contains(self.peek()) {
            // Try to parse a statement
            if let Some(stmt) = self.parse_stmt() {
                println!("Added statement to block: {:?}", stmt);
                stmts.push(stmt);
            } else {
                // If parsing failed, try to recover by skipping to the next statement
                println!("Error parsing statement in block, skipping to next statement");
                self.advance();
            }

            // Skip any whitespace tokens between statements
            self.skip_whitespace_tokens();

            // Prevent infinite loops by checking if we're at EOF
            if self.is_at_end() {
                println!("Reached end of file while parsing block");
                break;
            }
        }

        println!("Completed block with {} statements", stmts.len());
        Stmt::Block(stmts)
    }

    fn skip_whitespace_tokens(&mut self) {
        // In this implementation we don't have explicit whitespace tokens
        // But we can skip certain tokens if needed
        while matches!(self.peek(), &Token::EOF) && !self.is_at_end() {
            self.advance();
        }
    }

    fn parse_function_def(&mut self, gpu: bool) -> Option<Stmt> {
        self.advance(); // fn

        // Get function name
        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => {
                println!("Expected identifier after fn, found {:?}", token);
                return None;
            }
        };

        println!("Parsing function definition for '{}'", name);

        if !self.expect(Token::Lparen) {
            println!("Expected opening parenthesis after function name");
            return None;
        }

        // Parse parameters
        let mut params = Vec::new();
        while !self.check(&Token::Rparen) && !self.is_at_end() {
            let param_name = match self.advance() {
                Token::Identifier(id) => id,
                token => {
                    println!("Expected parameter name, found {:?}", token);
                    return None;
                }
            };

            let param_type = if self.match_token(&Token::Colon) {
                match self.advance() {
                    Token::Identifier(t) => Some(t),
                    token => {
                        println!("Expected parameter type, found {:?}", token);
                        return None;
                    }
                }
            } else {
                None
            };

            params.push((param_name, param_type));
            println!("Added parameter: {:?}", params.last().unwrap());

            if !self.check(&Token::Rparen) {
                if !self.expect(Token::Comma) {
                    println!("Expected comma or closing parenthesis after parameter");
                    return None;
                }
            }
        }

        if !self.expect(Token::Rparen) {
            println!("Expected closing parenthesis after parameters");
            return None;
        }

        // Parse return type if present
        let return_type = if self.match_token(&Token::Arrow) {
            match self.advance() {
                Token::Identifier(t) => {
                    println!("Function return type: {}", t);
                    Some(t)
                }
                token => {
                    println!("Expected return type, found {:?}", token);
                    return None;
                }
            }
        } else {
            None
        };

        if !self.expect(Token::Colon) {
            println!("Expected colon after function signature");
            return None;
        }

        println!("Parsing function body");
        let body = Box::new(self.parse_block());

        if gpu {
            println!("Defined GPU function '{}'", name);
        } else {
            println!("Defined CPU function '{}'", name);
        }

        Some(Stmt::FunctionDef {
            name,
            params,
            return_type,
            body,
            gpu,
        })
    }
}
