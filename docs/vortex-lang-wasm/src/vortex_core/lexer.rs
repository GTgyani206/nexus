// || shree ganesh ||

//this code will be consisting Lexer that will tokenize the input source code into tokens.
//The Lexer part --- the one that gives the parser, tokens to start analysing.
//GOAL: To know what characters are there
//this is the simple syntax of VORTEX : let mut x = 42 + 5
/*
    converting it into tokens:
    {
        Token::Let,
        Token::mut,
        Token::Identifier("x"),
        Token::Equal,
        Token::Number(42),
        Token::Plus,
        Token::Number(5),
    }
*/
use super::token::Token;
//S1: Defining tokens

// #[derive(Debug, Clone, PartialEq)]
// pub enum Token {
//     //Keywords
//     Let,
//     Mut,
//     If,
//     Then,
//     Else,
//     For,
//     In,
//     Range,
//     Fn,
//     Return,

//     //VORTEX mode
//     Branch,
//     Fallback,
//     Parallel,
//     GPU,

//     //Symbols
//     Equals,     // =
//     Plus,       // +
//     Minus,      // -
//     Star,       // *
//     Slash,      // /
//     Lparen,     // (
//     Rparen,     // )
//     LBrace,     // {
//     RBrace,     // }
//     Lsquare,    // [
//     Rsquare,    // ]
//     Dot,        // .
//     Comma,      // ,
//     Colon,      // :
//     Arrow,      // ->
//     FatArrow,   // =>

//     //Comparision sign
//     GT,         // >
//     LT,         // <
//     GE,         // >=
//     LE,         // <=
//     EQ,         // ==
//     NE,         // !=

//     // Literals
//     Number(i64),
//     Floating(f64),
//     String(String),
//     Boolean(bool),
//     Identifier(String),

//     //Other
//     EOF,
// }

// Lexer Structure
// This part would be scanning the code (as a whole) and then we'll be converting it into tokens in form of vectors
// one thing to be noted that the typical lexer does not store the tokens in a vector, but rather generates them on the fly as they are needed by the parser.
pub struct Lexer {
    input: Vec<char>, //tokens are extracted in loop then are append here
    position: usize,  //tells where in the source am I currently, and then the  advance() is called to move forward, This also ensures that the lexer does not process same character twice and also to keep it in bound
}

impl Lexer{
    pub fn new(input: &str) -> Self{
        Lexer{
            input: input.chars().collect(),
            position: 0,
        }
    }

    //to peek ahead and check if the next character is a certain character
    fn peek(&self) -> Option<char> {
        self.input.get(self.position).cloned()
    }

    fn peek_next(&self) -> Option<char> {
        self.input.get(self.position + 1).cloned()
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.input.get(self.position + n).cloned()
    }

    //move forward
    fn advance(&mut self) -> Option<char>{
        let ch = self.peek();
        self.position += 1;
        ch
    }

    fn skip_whitespace(&mut self){
        while let Some(ch) = self.peek(){
            if ch.is_whitespace() {
                self.advance();
            }else{
                break;
            }
        }
    }

    // Handle double dot for range operators (0..10)
    fn handle_range(&mut self) -> Token {
        self.advance(); // consume the first dot
        if self.peek() == Some('.') {
            self.advance(); // consume the second dot
            Token::Range
        } else {
            Token::Dot
        }
    }
    
    fn skip_comment(&mut self) {
        // Skip until the end of the line or end of file
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    //tokenizer logic
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        match self.advance(){

            Some('=') => {
                if self.peek() == Some('='){
                    self.advance();
                    Token::EQ
                }else if self.peek() == Some('>'){
                    self.advance();
                    Token::FatArrow
                }else{
                    Token::Equals
                }
            }

            Some('-') => {
                if self.peek() == Some('>'){
                    self.advance();
                    Token::Arrow
                }else {
                    Token::Minus
                }
            }

            Some('!') => {
                if self.peek() == Some('='){
                    self.advance();
                    Token::NE
                }else{
                    Token::EOF
                }
            }

            Some('>') => {
                if self.peek() == Some('='){
                    self.advance();
                    Token::GE
                }else{
                    Token::GT
                }
            }
            Some('<') => {
                if self.peek() == Some('='){
                    self.advance();
                    Token::LE
                }else{
                    Token::LT
                }
            }

            Some('+') => Token::Plus,
            Some('*') => Token::Star,
            Some('/') => {
                // Check for comments (// ...)
                if self.peek() == Some('/') {
                    self.advance(); // consume the second '/'
                    self.skip_comment();
                    return self.next_token(); // Get the next token after the comment
                } else {
                    Token::Slash
                }
            },
            Some(':') => Token::Colon,
            Some('.') => {
                if self.peek() == Some('.') {
                    self.advance(); // consume the second dot
                    Token::Range
                } else {
                    Token::Dot
                }
            },
            Some(',') => Token::Comma,
            Some('(') => Token::Lparen,
            Some(')') => Token::Rparen,
            Some('{') => Token::LBrace,
            Some('}') => Token::RBrace,
            Some('[') => Token::Lsquare,
            Some(']') => Token::Rsquare,

            Some('"') => {
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c == '"' {
                        self.advance();
                        break;
                    }else{
                        s.push(self.advance().unwrap());
                    }
                }
                Token::String(s)
            }

            Some(ch) if ch.is_ascii_digit() => {
                let mut number = ch.to_string();
                let mut is_float = false;

                while let Some(c) = self.peek(){
                    if c.is_ascii_digit(){
                        number.push(self.advance().unwrap());
                    }else if c == '.' && !is_float {
                        // Check if this is a range operator (..) not a decimal point
                        if self.peek_next() == Some('.') {
                            // This is a range operator, don't consume the dot
                            break;
                        }
                        is_float = true;
                        number.push(self.advance().unwrap());
                    }else{
                        break;
                    }
                }

                if is_float{
                    Token::Floating(number.parse::<f64>().unwrap())
                }else{
                    Token::Number(number.parse::<i64>().unwrap())
                }
            }

            Some('@') => {
                // Handle annotation tokens separately
                let mut ident = "@".to_string();
                while let Some(c) = self.peek(){
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(self.advance().unwrap());
                    }else{
                        break;
                    }
                }

                match ident.as_str() {
                    "@gpu" => Token::GPU,
                    _ => Token::Identifier(ident), // Unknown annotation
                }
            },
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let mut ident = ch.to_string();
                while let Some(c) = self.peek(){
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(self.advance().unwrap());
                    }else{
                        break;
                    }
                }

                match ident.as_str() {
                "let" => Token::Let,
                "mut" => Token::Mut,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "for" => Token::For,
                "in" => Token::In,
                "range" => Token::Range,
                "fn" => Token::Fn,
                "return" => Token::Return,
                "true" => Token::Boolean(true),
                "false" => Token::Boolean(false),
                "branch" => Token::Branch,
                "fallback" => Token::Fallback,
                "parallel" => Token::Parallel,
                "print" => Token::Identifier(ident), // Special case for print function
                _ => Token::Identifier(ident),
            }
        }
        Some(_) => Token::EOF, // Unknown char
        None => Token::EOF,
    }
    }
}
